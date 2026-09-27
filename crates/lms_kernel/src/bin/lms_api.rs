//! Minimal HTTP adapter for the first learner-registration slice.

use std::{collections::HashMap, env, net::SocketAddr, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{
        HeaderMap, HeaderValue, StatusCode,
        header::{AUTHORIZATION, WWW_AUTHENTICATE},
    },
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx_core::{error::Error as SqlxError, query::query, row::Row, transaction::Transaction};
use sqlx_postgres::{PgPool, PgPoolOptions, Postgres};
use thiserror::Error;
use uuid::Uuid;

const TENANT_API_KEY_SHA256_ENV: &str = "LMS_TENANT_API_KEY_SHA256";

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    tenant_authorizer: TenantAuthorizer,
}

#[derive(Clone)]
struct TenantAuthorizer {
    api_key_digests: Arc<HashMap<Uuid, [u8; 32]>>,
}

impl TenantAuthorizer {
    fn from_environment() -> Result<Self, ConfigurationError> {
        let raw = env::var(TENANT_API_KEY_SHA256_ENV)
            .map_err(|_| ConfigurationError::MissingTenantAuthorization)?;
        Self::from_json(&raw)
    }

    fn from_json(raw: &str) -> Result<Self, ConfigurationError> {
        let configured: HashMap<String, String> = serde_json::from_str(raw)
            .map_err(|_| ConfigurationError::InvalidTenantAuthorization)?;
        if configured.is_empty() {
            return Err(ConfigurationError::InvalidTenantAuthorization);
        }

        let mut api_key_digests = HashMap::with_capacity(configured.len());
        for (tenant_id, digest) in configured {
            let tenant_id = Uuid::parse_str(&tenant_id)
                .map_err(|_| ConfigurationError::InvalidTenantAuthorization)?;
            let digest =
                decode_sha256_hex(&digest).ok_or(ConfigurationError::InvalidTenantAuthorization)?;
            if api_key_digests.insert(tenant_id, digest).is_some() {
                return Err(ConfigurationError::InvalidTenantAuthorization);
            }
        }

        Ok(Self {
            api_key_digests: Arc::new(api_key_digests),
        })
    }

    fn authorize(&self, headers: &HeaderMap, tenant_id: Uuid) -> Result<(), ApiError> {
        let authorization = headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or(ApiError::Unauthorized)?;
        let (scheme, token) = authorization
            .split_once(' ')
            .ok_or(ApiError::Unauthorized)?;
        if !scheme.eq_ignore_ascii_case("bearer")
            || token.is_empty()
            || token.bytes().any(|byte| byte.is_ascii_whitespace())
        {
            return Err(ApiError::Unauthorized);
        }

        let expected_digest = self
            .api_key_digests
            .get(&tenant_id)
            .ok_or(ApiError::Forbidden)?;
        let supplied_digest: [u8; 32] = Sha256::digest(token.as_bytes()).into();
        if !constant_time_equal(expected_digest, &supplied_digest) {
            return Err(ApiError::Forbidden);
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
enum ConfigurationError {
    #[error("LMS_TENANT_API_KEY_SHA256 must be configured")]
    MissingTenantAuthorization,
    #[error("LMS_TENANT_API_KEY_SHA256 must be a non-empty tenant-to-SHA-256 JSON object")]
    InvalidTenantAuthorization,
    #[error(
        "the application database role must not be superuser, BYPASSRLS, a table owner, or a schema creator"
    )]
    UnsafeDatabaseRole,
}

#[derive(Debug)]
enum ApiError {
    Unauthorized,
    Forbidden,
    BadRequest(&'static str),
    Conflict,
    Database(SqlxError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let authenticate = matches!(&self, Self::Unauthorized);
        let (status, message) = match self {
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "bearer authorization is required"),
            Self::Forbidden => (StatusCode::FORBIDDEN, "tenant authorization failed"),
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            Self::Conflict => (StatusCode::CONFLICT, "resource already exists"),
            Self::Database(error) => {
                eprintln!("database request failed: {error}");
                (StatusCode::INTERNAL_SERVER_ERROR, "database request failed")
            }
        };
        let mut response = (status, Json(ErrorResponse { error: message })).into_response();
        if authenticate {
            response
                .headers_mut()
                .insert(WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        }
        response
    }
}

impl From<SqlxError> for ApiError {
    fn from(error: SqlxError) -> Self {
        if matches!(&error, SqlxError::Database(database_error) if database_error.code().as_deref() == Some("23505"))
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
fn router(pool: PgPool, tenant_authorizer: TenantAuthorizer) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/tenants/{tenant_id}/learners", post(create_learner))
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
        .with_state(AppState {
            pool,
            tenant_authorizer,
        })
}

async fn healthz(State(state): State<AppState>) -> Result<Json<HealthResponse>, ApiError> {
    query("SELECT 1").execute(&state.pool).await?;
    Ok(Json(HealthResponse { status: "ok" }))
}

fn validate_initial_membership_status(status: &str) -> Result<(), ApiError> {
    if status == "active" {
        Ok(())
    } else {
        Err(ApiError::BadRequest(
            "membership_status must be \"active\" for initial registration",
        ))
    }
}

async fn create_learner(
    State(state): State<AppState>,
    Path(tenant_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<CreateLearnerRequest>,
) -> Result<(StatusCode, Json<LearnerResponse>), ApiError> {
    state.tenant_authorizer.authorize(&headers, tenant_id)?;

    if tenant_id.is_nil()
        || request.identity_authority.trim().is_empty()
        || request.external_subject_reference.trim().is_empty()
    {
        return Err(ApiError::BadRequest(
            "tenant and identity references are required",
        ));
    }
    validate_initial_membership_status(&request.membership_status)?;

    let mut transaction = begin_tenant_transaction(&state.pool, tenant_id).await?;

    query(
        "INSERT INTO login_identity_reference (identity_authority, external_subject_reference) \
         VALUES ($1, $2) ON CONFLICT (identity_authority, external_subject_reference) DO NOTHING",
    )
    .bind(&request.identity_authority)
    .bind(&request.external_subject_reference)
    .execute(&mut *transaction)
    .await?;
    let identity = query(
        "SELECT login_identity_id FROM login_identity_reference \
         WHERE identity_authority = $1 AND external_subject_reference = $2",
    )
    .bind(&request.identity_authority)
    .bind(&request.external_subject_reference)
    .fetch_one(&mut *transaction)
    .await?;
    let login_identity_id: Uuid = identity.try_get("login_identity_id")?;

    query(
        "INSERT INTO learner_profile (login_identity_id) VALUES ($1) \
         ON CONFLICT (login_identity_id) DO NOTHING",
    )
    .bind(login_identity_id)
    .execute(&mut *transaction)
    .await?;
    let learner = query("SELECT learner_id FROM learner_profile WHERE login_identity_id = $1")
        .bind(login_identity_id)
        .fetch_one(&mut *transaction)
        .await?;
    let learner_id: Uuid = learner.try_get("learner_id")?;

    query(
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
    query("SELECT set_config('app.tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *transaction)
        .await?;
    Ok(transaction)
}

async fn create_offering(
    State(state): State<AppState>,
    Path(tenant_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<CreateOfferingRequest>,
) -> Result<(StatusCode, Json<OfferingResponse>), ApiError> {
    state.tenant_authorizer.authorize(&headers, tenant_id)?;

    if tenant_id.is_nil()
        || request.offering_name.trim().is_empty()
        || request.content_release_reference.trim().is_empty()
    {
        return Err(ApiError::BadRequest(
            "offering and content release references are required",
        ));
    }

    let mut transaction = begin_tenant_transaction(&state.pool, tenant_id).await?;
    let offering = query(
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
    headers: HeaderMap,
    Json(request): Json<CreateEntitlementRequest>,
) -> Result<(StatusCode, Json<EntitlementResponse>), ApiError> {
    state.tenant_authorizer.authorize(&headers, tenant_id)?;

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
    let entitlement = query(
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
    headers: HeaderMap,
    Json(request): Json<CreateEnrollmentRequest>,
) -> Result<(StatusCode, Json<EnrollmentResponse>), ApiError> {
    state.tenant_authorizer.authorize(&headers, tenant_id)?;

    if tenant_id.is_nil()
        || learner_id.is_nil()
        || request.course_offering_id.is_nil()
        || request.access_entitlement_id.is_nil()
    {
        return Err(ApiError::BadRequest("enrollment references are required"));
    }

    let mut transaction = begin_tenant_transaction(&state.pool, tenant_id).await?;
    let enrollment = query(
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
    headers: HeaderMap,
    Json(request): Json<CreateRegistrationRequest>,
) -> Result<(StatusCode, Json<RegistrationResponse>), ApiError> {
    state.tenant_authorizer.authorize(&headers, tenant_id)?;

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
    let registration = query(
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

fn decode_sha256_hex(value: &str) -> Option<[u8; 32]> {
    let bytes = value.as_bytes();
    if bytes.len() != 64 {
        return None;
    }

    let mut decoded = [0_u8; 32];
    for (index, byte) in decoded.iter_mut().enumerate() {
        let high = decode_hex_nibble(bytes[index * 2])?;
        let low = decode_hex_nibble(bytes[index * 2 + 1])?;
        *byte = (high << 4) | low;
    }
    Some(decoded)
}

fn decode_hex_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn constant_time_equal(left: &[u8; 32], right: &[u8; 32]) -> bool {
    left.iter()
        .zip(right)
        .fold(0_u8, |difference, (left, right)| {
            difference | (*left ^ *right)
        })
        == 0
}

async fn verify_application_role(pool: &PgPool) -> Result<(), ConfigurationError> {
    let role = query(
        "SELECT role.rolsuper, role.rolbypassrls, \
         EXISTS ( \
             SELECT 1 FROM pg_class relation \
             JOIN pg_namespace namespace ON namespace.oid = relation.relnamespace \
             WHERE relation.relowner = role.oid \
               AND relation.relkind IN ('r', 'p') \
               AND namespace.nspname = 'public' \
         ) AS owns_public_table, \
         has_schema_privilege(current_user, 'public', 'CREATE') AS can_create_in_public \
         FROM pg_roles role WHERE role.rolname = current_user",
    )
    .fetch_one(pool)
    .await
    .map_err(|_| ConfigurationError::UnsafeDatabaseRole)?;

    let is_superuser: bool = role
        .try_get("rolsuper")
        .map_err(|_| ConfigurationError::UnsafeDatabaseRole)?;
    let bypasses_rls: bool = role
        .try_get("rolbypassrls")
        .map_err(|_| ConfigurationError::UnsafeDatabaseRole)?;
    let owns_public_table: bool = role
        .try_get("owns_public_table")
        .map_err(|_| ConfigurationError::UnsafeDatabaseRole)?;
    let can_create_in_public: bool = role
        .try_get("can_create_in_public")
        .map_err(|_| ConfigurationError::UnsafeDatabaseRole)?;

    if is_superuser || bypasses_rls || owns_public_table || can_create_in_public {
        return Err(ConfigurationError::UnsafeDatabaseRole);
    }
    Ok(())
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
    let tenant_authorizer = TenantAuthorizer::from_environment()?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    verify_application_role(&pool).await?;
    let listener = tokio::net::TcpListener::bind(bind_address).await?;
    axum::serve(listener, router(pool, tenant_authorizer))
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOKEN: &str = "tenant-one-secret";
    const TOKEN_SHA256: &str = "31e5dddb43a81d8e349099d5c677503892dc8b79162e4f14ea36d933437140f7";

    fn tenant_id() -> Uuid {
        Uuid::from_u128(1)
    }

    fn authorizer() -> TenantAuthorizer {
        TenantAuthorizer::from_json(&format!(r#"{{"{}":"{TOKEN_SHA256}"}}"#, tenant_id()))
            .expect("valid authorization fixture")
    }

    fn bearer_headers(token: &'static str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            format!("Bearer {token}").parse().expect("valid header"),
        );
        headers
    }

    #[test]
    fn requires_bearer_authorization() {
        assert!(matches!(
            authorizer().authorize(&HeaderMap::new(), tenant_id()),
            Err(ApiError::Unauthorized)
        ));
    }

    #[test]
    fn rejects_a_valid_key_for_another_tenant() {
        assert!(matches!(
            authorizer().authorize(&bearer_headers(TOKEN), Uuid::from_u128(2)),
            Err(ApiError::Forbidden)
        ));
    }

    #[test]
    fn rejects_an_incorrect_tenant_key() {
        assert!(matches!(
            authorizer().authorize(&bearer_headers("wrong-secret"), tenant_id()),
            Err(ApiError::Forbidden)
        ));
    }

    #[test]
    fn accepts_the_key_bound_to_the_requested_tenant() {
        authorizer()
            .authorize(&bearer_headers(TOKEN), tenant_id())
            .expect("matching tenant key");
    }

    #[test]
    fn rejects_empty_or_malformed_authorization_configuration() {
        assert!(matches!(
            TenantAuthorizer::from_json("{}"),
            Err(ConfigurationError::InvalidTenantAuthorization)
        ));
        assert!(matches!(
            TenantAuthorizer::from_json(r#"{"not-a-tenant":"00"}"#),
            Err(ConfigurationError::InvalidTenantAuthorization)
        ));
    }

    #[test]
    fn initial_registration_requires_active_membership() {
        validate_initial_membership_status("active").expect("active registration is allowed");

        for status in ["suspended", "ended", "unknown", ""] {
            assert!(matches!(
                validate_initial_membership_status(status),
                Err(ApiError::BadRequest(
                    "membership_status must be \"active\" for initial registration"
                ))
            ));
        }
    }
}
