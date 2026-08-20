//! Minimal HTTP adapter for the first learner-registration slice.

use std::{env, net::SocketAddr};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Row, Transaction, postgres::PgPoolOptions};
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
            Self::Conflict => (StatusCode::CONFLICT, "resource already exists"),
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
        if matches!(&error, sqlx::Error::Database(database_error) if matches!(
            database_error.code().as_deref(),
            Some("23505") | Some("23P01")
        )) {
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

#[derive(Debug, Deserialize)]
struct CreateAffiliationRequest {
    affiliation_kind: String,
    valid_from: Option<DateTime<Utc>>,
    valid_to: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
struct AffiliationResponse {
    learning_affiliation_id: Uuid,
    tenant_id: Uuid,
    learner_id: Uuid,
    affiliation_kind: String,
    valid_from: DateTime<Utc>,
    valid_to: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
struct CreateOfferingRequest {
    offering_name: String,
    content_release_reference: String,
}

#[derive(Serialize)]
struct OfferingResponse {
    course_offering_id: Uuid,
    tenant_id: Uuid,
    offering_name: String,
    content_release_reference: String,
}

#[derive(Debug, Deserialize)]
struct CreateEntitlementRequest {
    source_authority: String,
    external_entitlement_reference: String,
    source_digest: String,
    source_version: String,
    valid_from: Option<DateTime<Utc>>,
    valid_to: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
struct EntitlementResponse {
    access_entitlement_id: Uuid,
    tenant_id: Uuid,
    learner_id: Uuid,
    valid_from: DateTime<Utc>,
    valid_to: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
struct CreateEnrollmentRequest {
    course_offering_id: Uuid,
    access_entitlement_id: Uuid,
}

#[derive(Serialize)]
struct EnrollmentResponse {
    enrollment_record_id: Uuid,
    tenant_id: Uuid,
    learner_id: Uuid,
    course_offering_id: Uuid,
    access_entitlement_id: Uuid,
    enrollment_status: &'static str,
}

#[derive(Debug, Deserialize)]
struct CreateRegistrationRequest {
    external_registration_reference: String,
}

#[derive(Serialize)]
struct RegistrationResponse {
    learning_registration_id: Uuid,
    tenant_id: Uuid,
    learner_id: Uuid,
    enrollment_record_id: Uuid,
    registration_status: &'static str,
}

/// Builds the HTTP router for the learner-registration adapter.
fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/tenants/{tenant_id}/learners", post(create_learner))
        .route(
            "/v1/tenants/{tenant_id}/learners/{learner_id}/affiliations",
            post(create_affiliation),
        )
        .route("/v1/tenants/{tenant_id}/offerings", post(create_offering))
        .route(
            "/v1/tenants/{tenant_id}/learners/{learner_id}/entitlements",
            post(create_entitlement),
        )
        .route(
            "/v1/tenants/{tenant_id}/learners/{learner_id}/enrollments",
            post(create_enrollment),
        )
        .route(
            "/v1/tenants/{tenant_id}/learners/{learner_id}/enrollments/{enrollment_record_id}/registrations",
            post(create_registration),
        )
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

    let mut transaction = begin_tenant_transaction(&state.pool, tenant_id).await?;

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

async fn begin_tenant_transaction(
    pool: &PgPool,
    tenant_id: Uuid,
) -> Result<Transaction<'_, Postgres>, ApiError> {
    let mut transaction = pool.begin().await?;
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *transaction)
        .await?;
    Ok(transaction)
}

async fn create_affiliation(
    State(state): State<AppState>,
    Path((tenant_id, learner_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<CreateAffiliationRequest>,
) -> Result<(StatusCode, Json<AffiliationResponse>), ApiError> {
    if tenant_id.is_nil() || learner_id.is_nil() || request.affiliation_kind.trim().is_empty() {
        return Err(ApiError::BadRequest("affiliation references are required"));
    }
    if !matches!(
        request.affiliation_kind.as_str(),
        "employee"
            | "contractor"
            | "partner"
            | "customer"
            | "candidate"
            | "student"
            | "guardian"
            | "association_member"
            | "public_learner"
            | "self_sponsored"
    ) {
        return Err(ApiError::BadRequest("affiliation kind is invalid"));
    }
    let valid_from = request.valid_from.unwrap_or_else(Utc::now);
    if request
        .valid_to
        .is_some_and(|valid_to| valid_to <= valid_from)
    {
        return Err(ApiError::BadRequest(
            "affiliation validity interval is invalid",
        ));
    }

    let mut transaction = begin_tenant_transaction(&state.pool, tenant_id).await?;
    let affiliation = sqlx::query(
        "INSERT INTO learning_affiliation \
         (tenant_id, learner_id, affiliation_kind, valid_from, valid_to) \
         SELECT $1, $2, $3, $4, $5 \
         FROM tenant_membership \
         WHERE tenant_id = $1 AND learner_id = $2 \
           AND membership_status = 'active' AND valid_from <= now() \
           AND (valid_to IS NULL OR valid_to > now()) \
         RETURNING learning_affiliation_id",
    )
    .bind(tenant_id)
    .bind(learner_id)
    .bind(&request.affiliation_kind)
    .bind(valid_from)
    .bind(request.valid_to)
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or(ApiError::BadRequest("learner membership is not active"))?;
    let learning_affiliation_id: Uuid = affiliation.try_get("learning_affiliation_id")?;
    transaction.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(AffiliationResponse {
            learning_affiliation_id,
            tenant_id,
            learner_id,
            affiliation_kind: request.affiliation_kind,
            valid_from,
            valid_to: request.valid_to,
        }),
    ))
}

async fn create_offering(
    State(state): State<AppState>,
    Path(tenant_id): Path<Uuid>,
    Json(request): Json<CreateOfferingRequest>,
) -> Result<(StatusCode, Json<OfferingResponse>), ApiError> {
    if tenant_id.is_nil()
        || request.offering_name.trim().is_empty()
        || request.content_release_reference.trim().is_empty()
    {
        return Err(ApiError::BadRequest(
            "offering and content release references are required",
        ));
    }

    let mut transaction = begin_tenant_transaction(&state.pool, tenant_id).await?;
    let offering = sqlx::query(
        "INSERT INTO course_offering \
         (tenant_id, offering_name, content_release_reference) \
         VALUES ($1, $2, $3) RETURNING course_offering_id",
    )
    .bind(tenant_id)
    .bind(&request.offering_name)
    .bind(&request.content_release_reference)
    .fetch_one(&mut *transaction)
    .await?;
    let course_offering_id: Uuid = offering.try_get("course_offering_id")?;
    transaction.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(OfferingResponse {
            course_offering_id,
            tenant_id,
            offering_name: request.offering_name,
            content_release_reference: request.content_release_reference,
        }),
    ))
}

async fn create_entitlement(
    State(state): State<AppState>,
    Path((tenant_id, learner_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<CreateEntitlementRequest>,
) -> Result<(StatusCode, Json<EntitlementResponse>), ApiError> {
    if tenant_id.is_nil()
        || learner_id.is_nil()
        || request.source_authority.trim().is_empty()
        || request.external_entitlement_reference.trim().is_empty()
        || request.source_digest.trim().is_empty()
        || request.source_version.trim().is_empty()
    {
        return Err(ApiError::BadRequest(
            "entitlement source metadata is required",
        ));
    }
    let valid_from = request.valid_from.unwrap_or_else(Utc::now);
    if request
        .valid_to
        .is_some_and(|valid_to| valid_to <= valid_from)
    {
        return Err(ApiError::BadRequest(
            "entitlement validity interval is invalid",
        ));
    }

    let mut transaction = begin_tenant_transaction(&state.pool, tenant_id).await?;
    let entitlement = sqlx::query(
        "INSERT INTO access_entitlement \
         (tenant_id, learner_id, source_authority, external_entitlement_reference, \
          source_digest, source_version, valid_from, valid_to) \
         SELECT $1, $2, $3, $4, $5, $6, $7, $8 \
         FROM tenant_membership \
         WHERE tenant_id = $1 AND learner_id = $2 \
           AND membership_status = 'active' AND valid_from <= now() \
           AND (valid_to IS NULL OR valid_to > now()) \
         RETURNING access_entitlement_id",
    )
    .bind(tenant_id)
    .bind(learner_id)
    .bind(&request.source_authority)
    .bind(&request.external_entitlement_reference)
    .bind(&request.source_digest)
    .bind(&request.source_version)
    .bind(valid_from)
    .bind(request.valid_to)
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or(ApiError::BadRequest("learner membership is not active"))?;
    let access_entitlement_id: Uuid = entitlement.try_get("access_entitlement_id")?;
    transaction.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(EntitlementResponse {
            access_entitlement_id,
            tenant_id,
            learner_id,
            valid_from,
            valid_to: request.valid_to,
        }),
    ))
}

async fn create_enrollment(
    State(state): State<AppState>,
    Path((tenant_id, learner_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<CreateEnrollmentRequest>,
) -> Result<(StatusCode, Json<EnrollmentResponse>), ApiError> {
    if tenant_id.is_nil()
        || learner_id.is_nil()
        || request.course_offering_id.is_nil()
        || request.access_entitlement_id.is_nil()
    {
        return Err(ApiError::BadRequest("enrollment references are required"));
    }

    let mut transaction = begin_tenant_transaction(&state.pool, tenant_id).await?;
    let enrollment = sqlx::query(
        "INSERT INTO enrollment_record \
         (tenant_id, learner_id, course_offering_id, access_entitlement_id) \
         SELECT $1, $2, offering.course_offering_id, entitlement.access_entitlement_id \
         FROM course_offering AS offering \
         JOIN access_entitlement AS entitlement \
           ON entitlement.tenant_id = offering.tenant_id \
          AND entitlement.access_entitlement_id = $4 \
          AND entitlement.learner_id = $2 \
          AND entitlement.valid_from <= now() \
          AND (entitlement.valid_to IS NULL OR entitlement.valid_to > now()) \
         JOIN tenant_membership AS membership \
           ON membership.tenant_id = offering.tenant_id \
          AND membership.learner_id = $2 \
          AND membership.membership_status = 'active' \
          AND membership.valid_from <= now() \
          AND (membership.valid_to IS NULL OR membership.valid_to > now()) \
         WHERE offering.tenant_id = $1 \
           AND offering.course_offering_id = $3 \
           AND offering.offering_status = 'active' \
         RETURNING enrollment_record_id",
    )
    .bind(tenant_id)
    .bind(learner_id)
    .bind(request.course_offering_id)
    .bind(request.access_entitlement_id)
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or(ApiError::BadRequest(
        "active offering, entitlement, and membership are required",
    ))?;
    let enrollment_record_id: Uuid = enrollment.try_get("enrollment_record_id")?;
    transaction.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(EnrollmentResponse {
            enrollment_record_id,
            tenant_id,
            learner_id,
            course_offering_id: request.course_offering_id,
            access_entitlement_id: request.access_entitlement_id,
            enrollment_status: "enrolled",
        }),
    ))
}

async fn create_registration(
    State(state): State<AppState>,
    Path((tenant_id, learner_id, enrollment_record_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(request): Json<CreateRegistrationRequest>,
) -> Result<(StatusCode, Json<RegistrationResponse>), ApiError> {
    if tenant_id.is_nil()
        || learner_id.is_nil()
        || enrollment_record_id.is_nil()
        || request.external_registration_reference.trim().is_empty()
    {
        return Err(ApiError::BadRequest(
            "enrollment and registration references are required",
        ));
    }

    let mut transaction = begin_tenant_transaction(&state.pool, tenant_id).await?;
    let registration = sqlx::query(
        "INSERT INTO learning_registration \
         (tenant_id, learner_id, enrollment_record_id, external_registration_reference) \
         SELECT tenant_id, learner_id, enrollment_record_id, $4 \
         FROM enrollment_record \
         WHERE tenant_id = $1 AND learner_id = $2 \
           AND enrollment_record_id = $3 AND enrollment_status = 'enrolled' \
         RETURNING learning_registration_id",
    )
    .bind(tenant_id)
    .bind(learner_id)
    .bind(enrollment_record_id)
    .bind(&request.external_registration_reference)
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or(ApiError::BadRequest("active enrollment is required"))?;
    let learning_registration_id: Uuid = registration.try_get("learning_registration_id")?;
    transaction.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(RegistrationResponse {
            learning_registration_id,
            tenant_id,
            learner_id,
            enrollment_record_id,
            registration_status: "registered",
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
