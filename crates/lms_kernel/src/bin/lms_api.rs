//! Minimal HTTP adapter for the first learner-registration slice.

use std::{env, net::SocketAddr};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row, postgres::PgPoolOptions};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    pool: PgPool,
}

#[derive(Debug)]
enum ApiError {
    BadRequest(&'static str),
    Conflict,
    Database(sqlx::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            Self::Conflict => (
                StatusCode::CONFLICT,
                "learner identity is already enrolled in this tenant",
            ),
            Self::Database(error) => {
                eprintln!("database request failed: {error}");
                (StatusCode::INTERNAL_SERVER_ERROR, "database request failed")
            }
        };
        (status, Json(ErrorResponse { error: message })).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        if matches!(&error, sqlx::Error::Database(database_error) if database_error.code().as_deref() == Some("23505"))
        {
            Self::Conflict
        } else {
            Self::Database(error)
        }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: &'static str,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Debug, Deserialize)]
struct CreateLearnerRequest {
    identity_authority: String,
    external_subject_reference: String,
    membership_status: String,
}

#[derive(Serialize)]
struct LearnerResponse {
    learner_id: Uuid,
    tenant_id: Uuid,
    identity_authority: String,
    external_subject_reference: String,
    membership_status: String,
}

/// Builds the HTTP router for the learner-registration adapter.
fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/tenants/{tenant_id}/learners", post(create_learner))
        .with_state(AppState { pool })
}

async fn healthz(State(state): State<AppState>) -> Result<Json<HealthResponse>, ApiError> {
    sqlx::query("SELECT 1").execute(&state.pool).await?;
    Ok(Json(HealthResponse { status: "ok" }))
}

async fn create_learner(
    State(state): State<AppState>,
    Path(tenant_id): Path<Uuid>,
    Json(request): Json<CreateLearnerRequest>,
) -> Result<(StatusCode, Json<LearnerResponse>), ApiError> {
    if tenant_id.is_nil()
        || request.identity_authority.trim().is_empty()
        || request.external_subject_reference.trim().is_empty()
    {
        return Err(ApiError::BadRequest(
            "tenant and identity references are required",
        ));
    }
    if !matches!(
        request.membership_status.as_str(),
        "active" | "suspended" | "ended"
    ) {
        return Err(ApiError::BadRequest("membership status is invalid"));
    }

    let mut transaction = state.pool.begin().await?;
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *transaction)
        .await?;

    sqlx::query(
        "INSERT INTO login_identity_reference (identity_authority, external_subject_reference) \
         VALUES ($1, $2) ON CONFLICT (identity_authority, external_subject_reference) DO NOTHING",
    )
    .bind(&request.identity_authority)
    .bind(&request.external_subject_reference)
    .execute(&mut *transaction)
    .await?;
    let identity = sqlx::query(
        "SELECT login_identity_id FROM login_identity_reference \
         WHERE identity_authority = $1 AND external_subject_reference = $2",
    )
    .bind(&request.identity_authority)
    .bind(&request.external_subject_reference)
    .fetch_one(&mut *transaction)
    .await?;
    let login_identity_id: Uuid = identity.try_get("login_identity_id")?;

    sqlx::query(
        "INSERT INTO learner_profile (login_identity_id) VALUES ($1) \
         ON CONFLICT (login_identity_id) DO NOTHING",
    )
    .bind(login_identity_id)
    .execute(&mut *transaction)
    .await?;
    let learner =
        sqlx::query("SELECT learner_id FROM learner_profile WHERE login_identity_id = $1")
            .bind(login_identity_id)
            .fetch_one(&mut *transaction)
            .await?;
    let learner_id: Uuid = learner.try_get("learner_id")?;

    sqlx::query(
        "INSERT INTO tenant_membership \
         (tenant_id, learner_id, membership_status, valid_from) \
         VALUES ($1, $2, $3, now())",
    )
    .bind(tenant_id)
    .bind(learner_id)
    .bind(&request.membership_status)
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(LearnerResponse {
            learner_id,
            tenant_id,
            identity_authority: request.identity_authority,
            external_subject_reference: request.external_subject_reference,
            membership_status: request.membership_status,
        }),
    ))
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL")?;
    let bind_address: SocketAddr = env::var("LMS_BIND_ADDRESS")
        .unwrap_or_else(|_| "127.0.0.1:8080".to_owned())
        .parse()?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    sqlx::migrate!("../../migrations").run(&pool).await?;
    let listener = tokio::net::TcpListener::bind(bind_address).await?;
    axum::serve(listener, router(pool))
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}
