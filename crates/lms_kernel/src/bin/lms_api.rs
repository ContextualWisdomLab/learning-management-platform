//! Minimal HTTP adapter for the first learner-registration slice.

use std::{
    collections::{BTreeSet, HashSet},
    env,
    net::SocketAddr,
};

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use lms_kernel::{
    AssessmentResultStatus, CompletionPolicyRevision, DecisionEvidenceReference, EvidenceKind,
    EvidenceSourceMetadata, KernelError, evaluate_completion,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
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

#[derive(Debug, Deserialize)]
struct CreateAttemptRequest {
    external_attempt_reference: String,
    content_release_reference: String,
}

#[derive(Serialize)]
struct AttemptResponse {
    learning_attempt_id: Uuid,
    tenant_id: Uuid,
    learner_id: Uuid,
    learning_registration_id: Uuid,
    attempt_status: &'static str,
}

#[derive(Debug, Deserialize)]
struct RecordProgressRequest {
    source_authority: String,
    external_activity_reference: String,
    source_version: String,
    source_digest: String,
    progress_state: String,
    progress_percent: f64,
    observed_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
struct ProgressResponse {
    progress_projection_id: Uuid,
    tenant_id: Uuid,
    learner_id: Uuid,
    learning_attempt_id: Uuid,
    progress_state: String,
    progress_percent: f64,
    observed_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct CreateCompletionPolicyRequest {
    policy_name: String,
}

#[derive(Serialize)]
struct CompletionPolicyResponse {
    completion_policy_id: Uuid,
    tenant_id: Uuid,
    policy_name: String,
}

#[derive(Debug, Deserialize)]
struct CreatePolicyRevisionRequest {
    revision_number: i32,
    required_evidence_kinds: Vec<String>,
    published_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
struct PolicyRevisionResponse {
    completion_policy_revision_id: Uuid,
    tenant_id: Uuid,
    completion_policy_id: Uuid,
    revision_number: i32,
    required_evidence_kinds: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CreateEvidenceRequest {
    evidence_kind: String,
    source_authority: String,
    source_snapshot_reference: String,
    source_digest: String,
    source_version: String,
    assessment_result_status: Option<String>,
    observed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
struct CreateAssessmentResultRequest {
    contract_version: String,
    assessment_authority: String,
    external_result_reference: String,
    source_digest: String,
    source_version: String,
    assessment_result_status: String,
    idempotency_key: String,
    observed_at: DateTime<Utc>,
}

#[derive(Serialize)]
struct EvidenceResponse {
    decision_evidence_reference_id: Uuid,
    tenant_id: Uuid,
    learner_id: Uuid,
    evidence_kind: String,
    assessment_result_status: Option<String>,
    idempotency_key: Option<String>,
    observed_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct CreateCompletionDecisionRequest {
    completion_policy_revision_id: Uuid,
    evidence_reference_ids: Vec<Uuid>,
    evaluated_at: Option<DateTime<Utc>>,
    supersedes_decision_id: Option<Uuid>,
}

#[derive(Serialize)]
struct CompletionDecisionResponse {
    completion_decision_id: Uuid,
    tenant_id: Uuid,
    learner_id: Uuid,
    learning_registration_id: Uuid,
    supersedes_decision_id: Option<Uuid>,
    replay_fingerprint: String,
    evaluated_at: DateTime<Utc>,
    completion_status: &'static str,
}

#[derive(Debug, Deserialize)]
struct CreateCredentialRequest {
    credential_authority: String,
    external_credential_reference: String,
}

#[derive(Serialize)]
struct CredentialResponse {
    credential_record_id: Uuid,
    tenant_id: Uuid,
    learner_id: Uuid,
    learning_registration_id: Uuid,
    completion_decision_id: Uuid,
    credential_authority: String,
    external_credential_reference: String,
    credential_status: String,
    issued_at: DateTime<Utc>,
    revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
struct AuditEventQuery {
    limit: Option<i64>,
}

#[derive(Serialize)]
struct AuditEventResponse {
    audit_event_record_id: Uuid,
    tenant_id: Uuid,
    correlation_id: Uuid,
    actor_authority: String,
    actor_subject_reference: String,
    action_name: String,
    entity_type: String,
    entity_id: Uuid,
    source_authority: String,
    source_version: String,
    event_digest: String,
    occurred_at: DateTime<Utc>,
}

const ASSESSMENT_RESULT_REFERENCE_CONTRACT: &str = "assessment_result_reference/v1";

/// Builds the HTTP router for the learner-registration adapter.
fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/tenants/{tenant_id}/audit-events", get(list_audit_events))
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
        .route(
            "/v1/tenants/{tenant_id}/learners/{learner_id}/registrations/{learning_registration_id}/attempts",
            post(create_attempt),
        )
        .route(
            "/v1/tenants/{tenant_id}/learners/{learner_id}/attempts/{learning_attempt_id}/progress",
            post(record_progress),
        )
        .route(
            "/v1/tenants/{tenant_id}/completion-policies",
            post(create_completion_policy),
        )
        .route(
            "/v1/tenants/{tenant_id}/completion-policies/{completion_policy_id}/revisions",
            post(create_policy_revision),
        )
        .route(
            "/v1/tenants/{tenant_id}/learners/{learner_id}/registrations/{learning_registration_id}/evidence",
            post(create_evidence),
        )
        .route(
            "/v1/tenants/{tenant_id}/learners/{learner_id}/registrations/{learning_registration_id}/assessment-results",
            post(create_assessment_result),
        )
        .route(
            "/v1/tenants/{tenant_id}/learners/{learner_id}/registrations/{learning_registration_id}/completion-decisions",
            post(create_completion_decision),
        )
        .route(
            "/v1/tenants/{tenant_id}/learners/{learner_id}/registrations/{learning_registration_id}/completion-decisions/{completion_decision_id}/credentials",
            post(create_credential),
        )
        .route(
            "/v1/tenants/{tenant_id}/learners/{learner_id}/registrations/{learning_registration_id}/completion-decisions/{completion_decision_id}/credentials/{credential_record_id}/revoke",
            post(revoke_credential),
        )
        .with_state(AppState { pool })
}

async fn healthz(State(state): State<AppState>) -> Result<Json<HealthResponse>, ApiError> {
    sqlx::query("SELECT 1").execute(&state.pool).await?;
    Ok(Json(HealthResponse { status: "ok" }))
}

async fn list_audit_events(
    State(state): State<AppState>,
    Path(tenant_id): Path<Uuid>,
    Query(query): Query<AuditEventQuery>,
) -> Result<Json<Vec<AuditEventResponse>>, ApiError> {
    if tenant_id.is_nil() {
        return Err(ApiError::BadRequest("tenant reference is required"));
    }
    let limit = query.limit.unwrap_or(100);
    if !(1..=1000).contains(&limit) {
        return Err(ApiError::BadRequest(
            "audit event limit must be between 1 and 1000",
        ));
    }

    let mut transaction = begin_tenant_transaction(&state.pool, tenant_id).await?;
    let rows = sqlx::query(
        "SELECT audit_event_record_id, tenant_id, correlation_id, actor_authority, \
                actor_subject_reference, action_name, entity_type, entity_id, \
                source_authority, source_version, event_digest, occurred_at \
         FROM audit_event_record \
         WHERE tenant_id = $1 \
         ORDER BY occurred_at, audit_event_record_id \
         LIMIT $2",
    )
    .bind(tenant_id)
    .bind(limit)
    .fetch_all(&mut *transaction)
    .await?;
    let events = rows
        .into_iter()
        .map(|row| {
            Ok(AuditEventResponse {
                audit_event_record_id: row.try_get("audit_event_record_id")?,
                tenant_id: row.try_get("tenant_id")?,
                correlation_id: row.try_get("correlation_id")?,
                actor_authority: row.try_get("actor_authority")?,
                actor_subject_reference: row.try_get("actor_subject_reference")?,
                action_name: row.try_get("action_name")?,
                entity_type: row.try_get("entity_type")?,
                entity_id: row.try_get("entity_id")?,
                source_authority: row.try_get("source_authority")?,
                source_version: row.try_get("source_version")?,
                event_digest: row.try_get("event_digest")?,
                occurred_at: row.try_get("occurred_at")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;
    transaction.commit().await?;

    Ok(Json(events))
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

fn request_correlation_id(headers: &HeaderMap) -> Result<Uuid, ApiError> {
    headers
        .get("x-correlation-id")
        .map(|value| {
            value
                .to_str()
                .ok()
                .and_then(|value| Uuid::parse_str(value).ok())
                .ok_or(ApiError::BadRequest("correlation ID must be a UUID"))
        })
        .unwrap_or_else(|| Ok(Uuid::new_v4()))
}

struct AuditEvent<'a> {
    tenant_id: Uuid,
    correlation_id: Uuid,
    action_name: &'a str,
    entity_type: &'a str,
    entity_id: Uuid,
    source_authority: &'a str,
    source_version: &'a str,
    occurred_at: DateTime<Utc>,
}

fn audit_event_digest(event: &AuditEvent<'_>) -> String {
    let mut hasher = Sha256::new();
    for part in [
        event.tenant_id.to_string(),
        event.correlation_id.to_string(),
        event.action_name.to_owned(),
        event.entity_type.to_owned(),
        event.entity_id.to_string(),
        event.source_authority.to_owned(),
        event.source_version.to_owned(),
        event.occurred_at.to_rfc3339(),
    ] {
        hasher.update(part.as_bytes());
        hasher.update([0_u8]);
    }
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

async fn record_audit_event(
    transaction: &mut Transaction<'_, Postgres>,
    event: AuditEvent<'_>,
) -> Result<(), ApiError> {
    let event_digest = audit_event_digest(&event);
    sqlx::query(
        "INSERT INTO audit_event_record \
         (tenant_id, correlation_id, action_name, entity_type, entity_id, \
          source_authority, source_version, event_digest, occurred_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(event.tenant_id)
    .bind(event.correlation_id)
    .bind(event.action_name)
    .bind(event.entity_type)
    .bind(event.entity_id)
    .bind(event.source_authority)
    .bind(event.source_version)
    .bind(event_digest)
    .bind(event.occurred_at)
    .execute(&mut **transaction)
    .await?;
    Ok(())
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

async fn create_attempt(
    State(state): State<AppState>,
    Path((tenant_id, learner_id, learning_registration_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(request): Json<CreateAttemptRequest>,
) -> Result<(StatusCode, Json<AttemptResponse>), ApiError> {
    if tenant_id.is_nil()
        || learner_id.is_nil()
        || learning_registration_id.is_nil()
        || request.external_attempt_reference.trim().is_empty()
        || request.content_release_reference.trim().is_empty()
    {
        return Err(ApiError::BadRequest("attempt references are required"));
    }

    let mut transaction = begin_tenant_transaction(&state.pool, tenant_id).await?;
    let attempt = sqlx::query(
        "INSERT INTO learning_attempt \
         (tenant_id, learner_id, learning_registration_id, external_attempt_reference, \
          content_release_reference) \
         SELECT $1, $2, $3, $4, $5 \
         FROM learning_registration \
         WHERE tenant_id = $1 AND learner_id = $2 \
           AND learning_registration_id = $3 \
           AND registration_status IN ('registered', 'launched') \
         RETURNING learning_attempt_id",
    )
    .bind(tenant_id)
    .bind(learner_id)
    .bind(learning_registration_id)
    .bind(&request.external_attempt_reference)
    .bind(&request.content_release_reference)
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or(ApiError::BadRequest(
        "active learning registration is required",
    ))?;
    let learning_attempt_id: Uuid = attempt.try_get("learning_attempt_id")?;
    sqlx::query(
        "UPDATE learning_registration SET registration_status = 'launched' \
         WHERE tenant_id = $1 AND learner_id = $2 AND learning_registration_id = $3",
    )
    .bind(tenant_id)
    .bind(learner_id)
    .bind(learning_registration_id)
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(AttemptResponse {
            learning_attempt_id,
            tenant_id,
            learner_id,
            learning_registration_id,
            attempt_status: "launched",
        }),
    ))
}

async fn record_progress(
    State(state): State<AppState>,
    Path((tenant_id, learner_id, learning_attempt_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(request): Json<RecordProgressRequest>,
) -> Result<(StatusCode, Json<ProgressResponse>), ApiError> {
    if tenant_id.is_nil()
        || learner_id.is_nil()
        || learning_attempt_id.is_nil()
        || request.source_authority.trim().is_empty()
        || request.external_activity_reference.trim().is_empty()
        || request.source_version.trim().is_empty()
        || request.source_digest.trim().is_empty()
    {
        return Err(ApiError::BadRequest("progress source metadata is required"));
    }
    if !matches!(
        request.progress_state.as_str(),
        "not_started" | "in_progress" | "completed"
    ) || !request.progress_percent.is_finite()
        || !(0.0..=100.0).contains(&request.progress_percent)
    {
        return Err(ApiError::BadRequest("progress value is invalid"));
    }
    let observed_at = request.observed_at.unwrap_or_else(Utc::now);

    let mut transaction = begin_tenant_transaction(&state.pool, tenant_id).await?;
    let projection = sqlx::query(
        "INSERT INTO progress_projection \
         (tenant_id, learner_id, learning_attempt_id, source_authority, \
          external_activity_reference, source_version, source_digest, progress_state, \
          progress_percent, observed_at) \
         SELECT $1, $2, $3, $4, $5, $6, $7, $8, $9, $10 \
         FROM learning_attempt \
         WHERE tenant_id = $1 AND learner_id = $2 AND learning_attempt_id = $3 \
           AND attempt_status IN ('launched', 'active') \
         ON CONFLICT (tenant_id, learning_attempt_id, source_authority, \
                      external_activity_reference, source_version) DO UPDATE \
         SET source_digest = EXCLUDED.source_digest, \
             progress_state = EXCLUDED.progress_state, \
             progress_percent = EXCLUDED.progress_percent, \
             observed_at = EXCLUDED.observed_at, \
             recorded_at = now() \
         WHERE progress_projection.observed_at <= EXCLUDED.observed_at \
         RETURNING progress_projection_id",
    )
    .bind(tenant_id)
    .bind(learner_id)
    .bind(learning_attempt_id)
    .bind(&request.source_authority)
    .bind(&request.external_activity_reference)
    .bind(&request.source_version)
    .bind(&request.source_digest)
    .bind(&request.progress_state)
    .bind(request.progress_percent)
    .bind(observed_at)
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or(ApiError::BadRequest(
        "active learning attempt or newer progress observation is required",
    ))?;
    let progress_projection_id: Uuid = projection.try_get("progress_projection_id")?;
    let next_attempt_status = if request.progress_state == "completed" {
        "completed"
    } else {
        "active"
    };
    sqlx::query(
        "UPDATE learning_attempt SET attempt_status = $4, \
         closed_at = CASE WHEN $4 = 'completed' THEN COALESCE(closed_at, now()) ELSE NULL END \
         WHERE tenant_id = $1 AND learner_id = $2 AND learning_attempt_id = $3",
    )
    .bind(tenant_id)
    .bind(learner_id)
    .bind(learning_attempt_id)
    .bind(next_attempt_status)
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(ProgressResponse {
            progress_projection_id,
            tenant_id,
            learner_id,
            learning_attempt_id,
            progress_state: request.progress_state,
            progress_percent: request.progress_percent,
            observed_at,
        }),
    ))
}

fn parse_evidence_kind(value: &str) -> Result<EvidenceKind, ApiError> {
    match value {
        "activity" => Ok(EvidenceKind::Activity),
        "assessment" => Ok(EvidenceKind::Assessment),
        "attendance" => Ok(EvidenceKind::Attendance),
        "entitlement" => Ok(EvidenceKind::Entitlement),
        _ => Err(ApiError::BadRequest("evidence kind is invalid")),
    }
}

fn evidence_kind_name(kind: &EvidenceKind) -> &'static str {
    match kind {
        EvidenceKind::Activity => "activity",
        EvidenceKind::Assessment => "assessment",
        EvidenceKind::Attendance => "attendance",
        EvidenceKind::Entitlement => "entitlement",
    }
}

fn parse_assessment_result_status(value: &str) -> Result<AssessmentResultStatus, ApiError> {
    match value {
        "passed" => Ok(AssessmentResultStatus::Passed),
        "failed" => Ok(AssessmentResultStatus::Failed),
        "inconclusive" => Ok(AssessmentResultStatus::Inconclusive),
        _ => Err(ApiError::BadRequest("assessment result status is invalid")),
    }
}

fn assessment_result_status_name(status: &AssessmentResultStatus) -> &'static str {
    match status {
        AssessmentResultStatus::Passed => "passed",
        AssessmentResultStatus::Failed => "failed",
        AssessmentResultStatus::Inconclusive => "inconclusive",
    }
}

fn map_kernel_error(error: KernelError) -> ApiError {
    match error {
        KernelError::IncompleteEvidence => {
            ApiError::BadRequest("completion requirements are not satisfied")
        }
        KernelError::BoundaryMismatch => ApiError::BadRequest("completion boundary is invalid"),
        KernelError::DuplicateEvidence => {
            ApiError::BadRequest("evidence references are duplicated")
        }
        KernelError::MissingAssessmentResultStatus => {
            ApiError::BadRequest("assessment result status is required")
        }
        KernelError::AssessmentResultStatusNotAllowed => {
            ApiError::BadRequest("assessment result status is only valid for assessment evidence")
        }
        KernelError::AssessmentResultNotPassed => {
            ApiError::BadRequest("assessment result is not passed")
        }
        _ => ApiError::BadRequest("completion input is invalid"),
    }
}

async fn create_completion_policy(
    State(state): State<AppState>,
    Path(tenant_id): Path<Uuid>,
    Json(request): Json<CreateCompletionPolicyRequest>,
) -> Result<(StatusCode, Json<CompletionPolicyResponse>), ApiError> {
    if tenant_id.is_nil() || request.policy_name.trim().is_empty() {
        return Err(ApiError::BadRequest("completion policy name is required"));
    }
    let mut transaction = begin_tenant_transaction(&state.pool, tenant_id).await?;
    let policy = sqlx::query(
        "INSERT INTO completion_policy (tenant_id, policy_name) \
         VALUES ($1, $2) RETURNING completion_policy_id",
    )
    .bind(tenant_id)
    .bind(&request.policy_name)
    .fetch_one(&mut *transaction)
    .await?;
    let completion_policy_id: Uuid = policy.try_get("completion_policy_id")?;
    transaction.commit().await?;
    Ok((
        StatusCode::CREATED,
        Json(CompletionPolicyResponse {
            completion_policy_id,
            tenant_id,
            policy_name: request.policy_name,
        }),
    ))
}

async fn create_policy_revision(
    State(state): State<AppState>,
    Path((tenant_id, completion_policy_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<CreatePolicyRevisionRequest>,
) -> Result<(StatusCode, Json<PolicyRevisionResponse>), ApiError> {
    if tenant_id.is_nil() || completion_policy_id.is_nil() || request.revision_number <= 0 {
        return Err(ApiError::BadRequest(
            "policy revision references are invalid",
        ));
    }
    let mut required_evidence_kinds = BTreeSet::new();
    for kind in &request.required_evidence_kinds {
        if !required_evidence_kinds.insert(parse_evidence_kind(kind)?) {
            return Err(ApiError::BadRequest("policy evidence kinds must be unique"));
        }
    }
    if required_evidence_kinds.is_empty() {
        return Err(ApiError::BadRequest("policy evidence kinds are required"));
    }
    CompletionPolicyRevision::new(
        tenant_id,
        completion_policy_id,
        request.revision_number as u32,
        required_evidence_kinds.clone(),
    )
    .map_err(map_kernel_error)?;
    let required_json = serde_json::to_value(&required_evidence_kinds)
        .map_err(|_| ApiError::BadRequest("policy evidence kinds are invalid"))?;
    let published_at = request.published_at.unwrap_or_else(Utc::now);

    let mut transaction = begin_tenant_transaction(&state.pool, tenant_id).await?;
    let revision = sqlx::query(
        "INSERT INTO completion_policy_revision \
         (tenant_id, completion_policy_id, revision_number, required_evidence_kinds, published_at) \
         SELECT $1, $2, $3, $4, $5 \
         FROM completion_policy \
         WHERE tenant_id = $1 AND completion_policy_id = $2 \
         RETURNING completion_policy_revision_id",
    )
    .bind(tenant_id)
    .bind(completion_policy_id)
    .bind(request.revision_number)
    .bind(required_json)
    .bind(published_at)
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or(ApiError::BadRequest(
        "completion policy is not in this tenant",
    ))?;
    let completion_policy_revision_id: Uuid = revision.try_get("completion_policy_revision_id")?;
    transaction.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(PolicyRevisionResponse {
            completion_policy_revision_id,
            tenant_id,
            completion_policy_id,
            revision_number: request.revision_number,
            required_evidence_kinds: request.required_evidence_kinds,
        }),
    ))
}

struct EvidenceInsertRequest<'a> {
    tenant_id: Uuid,
    learner_id: Uuid,
    learning_registration_id: Uuid,
    evidence_kind: &'a str,
    source_metadata: &'a EvidenceSourceMetadata,
    assessment_result_status: Option<&'a str>,
    idempotency_key: Option<&'a str>,
    observed_at: DateTime<Utc>,
}

async fn insert_evidence_reference(
    transaction: &mut Transaction<'_, Postgres>,
    request: EvidenceInsertRequest<'_>,
) -> Result<(Uuid, bool), ApiError> {
    let evidence = sqlx::query(
        "INSERT INTO decision_evidence_reference \
         (tenant_id, learner_id, learning_registration_id, evidence_kind, source_authority, \
          source_snapshot_reference, source_digest, source_version, assessment_result_status, \
          idempotency_key, observed_at) \
         SELECT $1, $2, $11, $3, $4, $5, $6, $7, $8, $9, $10 \
         FROM learning_registration \
         WHERE tenant_id = $1 AND learner_id = $2 AND learning_registration_id = $11 \
           AND registration_status IN ('registered', 'launched') \
         ON CONFLICT (tenant_id, source_authority, idempotency_key) DO NOTHING \
         RETURNING decision_evidence_reference_id",
    )
    .bind(request.tenant_id)
    .bind(request.learner_id)
    .bind(request.evidence_kind)
    .bind(&request.source_metadata.source_authority)
    .bind(&request.source_metadata.source_snapshot_reference)
    .bind(&request.source_metadata.source_digest)
    .bind(&request.source_metadata.source_version)
    .bind(request.assessment_result_status)
    .bind(request.idempotency_key)
    .bind(request.observed_at)
    .bind(request.learning_registration_id)
    .fetch_optional(&mut **transaction)
    .await?;
    if let Some(evidence) = evidence {
        return Ok((evidence.try_get("decision_evidence_reference_id")?, true));
    }

    let idempotency_key = request.idempotency_key.ok_or(ApiError::Conflict)?;
    let existing = sqlx::query(
        "SELECT decision_evidence_reference_id, learner_id, learning_registration_id, \
                evidence_kind, source_snapshot_reference, source_digest, source_version, \
                assessment_result_status \
         FROM decision_evidence_reference \
         WHERE tenant_id = $1 AND source_authority = $2 AND idempotency_key = $3",
    )
    .bind(request.tenant_id)
    .bind(&request.source_metadata.source_authority)
    .bind(idempotency_key)
    .fetch_optional(&mut **transaction)
    .await?
    .ok_or(ApiError::Conflict)?;
    let existing_assessment_status: Option<String> =
        existing.try_get("assessment_result_status")?;
    let same_request = existing.try_get::<Uuid, _>("learner_id")? == request.learner_id
        && existing.try_get::<Uuid, _>("learning_registration_id")?
            == request.learning_registration_id
        && existing.try_get::<String, _>("evidence_kind")? == request.evidence_kind
        && existing.try_get::<String, _>("source_snapshot_reference")?
            == request.source_metadata.source_snapshot_reference
        && existing.try_get::<String, _>("source_digest")? == request.source_metadata.source_digest
        && existing.try_get::<String, _>("source_version")?
            == request.source_metadata.source_version
        && existing_assessment_status.as_deref() == request.assessment_result_status;
    if !same_request {
        return Err(ApiError::Conflict);
    }
    Ok((existing.try_get("decision_evidence_reference_id")?, false))
}

async fn create_evidence(
    State(state): State<AppState>,
    Path((tenant_id, learner_id, learning_registration_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(request): Json<CreateEvidenceRequest>,
) -> Result<(StatusCode, Json<EvidenceResponse>), ApiError> {
    if tenant_id.is_nil()
        || learner_id.is_nil()
        || learning_registration_id.is_nil()
        || request.source_authority.trim().is_empty()
        || request.source_snapshot_reference.trim().is_empty()
        || request.source_digest.trim().is_empty()
        || request.source_version.trim().is_empty()
    {
        return Err(ApiError::BadRequest("evidence source metadata is required"));
    }
    let evidence_kind = parse_evidence_kind(&request.evidence_kind)?;
    let assessment_result_status = request
        .assessment_result_status
        .as_deref()
        .map(parse_assessment_result_status)
        .transpose()?;
    let source_metadata = EvidenceSourceMetadata::new(
        &request.source_authority,
        &request.source_snapshot_reference,
        &request.source_digest,
        &request.source_version,
    )
    .map_err(map_kernel_error)?;
    let observed_at = request.observed_at.unwrap_or_else(Utc::now);
    DecisionEvidenceReference::new_with_assessment_status(
        tenant_id,
        learner_id,
        evidence_kind.clone(),
        source_metadata.clone(),
        observed_at,
        assessment_result_status,
    )
    .map_err(map_kernel_error)?;
    let assessment_result_status_name = assessment_result_status
        .as_ref()
        .map(|status| assessment_result_status_name(status).to_owned());
    let mut transaction = begin_tenant_transaction(&state.pool, tenant_id).await?;
    let (decision_evidence_reference_id, _) = insert_evidence_reference(
        &mut transaction,
        EvidenceInsertRequest {
            tenant_id,
            learner_id,
            learning_registration_id,
            evidence_kind: evidence_kind_name(&evidence_kind),
            source_metadata: &source_metadata,
            assessment_result_status: assessment_result_status_name.as_deref(),
            idempotency_key: None,
            observed_at,
        },
    )
    .await?;
    transaction.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(EvidenceResponse {
            decision_evidence_reference_id,
            tenant_id,
            learner_id,
            evidence_kind: evidence_kind_name(&evidence_kind).to_owned(),
            assessment_result_status: assessment_result_status_name,
            idempotency_key: None,
            observed_at,
        }),
    ))
}

async fn create_assessment_result(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((tenant_id, learner_id, learning_registration_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(request): Json<CreateAssessmentResultRequest>,
) -> Result<(StatusCode, Json<EvidenceResponse>), ApiError> {
    if tenant_id.is_nil()
        || learner_id.is_nil()
        || learning_registration_id.is_nil()
        || request.contract_version != ASSESSMENT_RESULT_REFERENCE_CONTRACT
        || request.assessment_authority.trim().is_empty()
        || request.external_result_reference.trim().is_empty()
        || request.source_digest.trim().is_empty()
        || request.source_version.trim().is_empty()
        || request.idempotency_key.trim().is_empty()
    {
        return Err(ApiError::BadRequest(
            "assessment result contract and source metadata are required",
        ));
    }
    let assessment_result_status =
        parse_assessment_result_status(&request.assessment_result_status)?;
    let source_metadata = EvidenceSourceMetadata::new(
        &request.assessment_authority,
        &request.external_result_reference,
        &request.source_digest,
        &request.source_version,
    )
    .map_err(map_kernel_error)?;
    let observed_at = request.observed_at;
    DecisionEvidenceReference::new_with_assessment_status(
        tenant_id,
        learner_id,
        EvidenceKind::Assessment,
        source_metadata.clone(),
        observed_at,
        Some(assessment_result_status),
    )
    .map_err(map_kernel_error)?;
    let status_name = assessment_result_status_name(&assessment_result_status).to_owned();
    let correlation_id = request_correlation_id(&headers)?;
    let mut transaction = begin_tenant_transaction(&state.pool, tenant_id).await?;
    let (decision_evidence_reference_id, inserted) = insert_evidence_reference(
        &mut transaction,
        EvidenceInsertRequest {
            tenant_id,
            learner_id,
            learning_registration_id,
            evidence_kind: evidence_kind_name(&EvidenceKind::Assessment),
            source_metadata: &source_metadata,
            assessment_result_status: Some(&status_name),
            idempotency_key: Some(&request.idempotency_key),
            observed_at,
        },
    )
    .await?;
    if inserted {
        record_audit_event(
            &mut transaction,
            AuditEvent {
                tenant_id,
                correlation_id,
                action_name: "assessment_result_reference.recorded",
                entity_type: "decision_evidence_reference",
                entity_id: decision_evidence_reference_id,
                source_authority: &request.assessment_authority,
                source_version: &request.source_version,
                occurred_at: observed_at,
            },
        )
        .await?;
    }
    transaction.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(EvidenceResponse {
            decision_evidence_reference_id,
            tenant_id,
            learner_id,
            evidence_kind: "assessment".to_owned(),
            assessment_result_status: Some(status_name),
            idempotency_key: Some(request.idempotency_key),
            observed_at,
        }),
    ))
}

async fn create_completion_decision(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((tenant_id, learner_id, learning_registration_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(request): Json<CreateCompletionDecisionRequest>,
) -> Result<(StatusCode, Json<CompletionDecisionResponse>), ApiError> {
    if tenant_id.is_nil()
        || learner_id.is_nil()
        || learning_registration_id.is_nil()
        || request.completion_policy_revision_id.is_nil()
        || request.evidence_reference_ids.is_empty()
    {
        return Err(ApiError::BadRequest("completion references are required"));
    }
    let mut seen = HashSet::new();
    if request
        .evidence_reference_ids
        .iter()
        .any(|evidence_id| evidence_id.is_nil() || !seen.insert(*evidence_id))
    {
        return Err(ApiError::BadRequest(
            "evidence references must be unique and non-nil",
        ));
    }
    let evaluated_at = request.evaluated_at.unwrap_or_else(Utc::now);
    let correlation_id = request_correlation_id(&headers)?;
    let mut transaction = begin_tenant_transaction(&state.pool, tenant_id).await?;
    if let Some(supersedes_decision_id) = request.supersedes_decision_id {
        if supersedes_decision_id.is_nil() {
            return Err(ApiError::BadRequest(
                "superseded decision reference is invalid",
            ));
        }
        sqlx::query(
            "SELECT completion_decision_id FROM completion_decision \
             WHERE tenant_id = $1 AND learner_id = $2 \
               AND learning_registration_id = $3 AND completion_decision_id = $4",
        )
        .bind(tenant_id)
        .bind(learner_id)
        .bind(learning_registration_id)
        .bind(supersedes_decision_id)
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(ApiError::BadRequest(
            "superseded decision is not in this registration",
        ))?;
    }
    let revision = sqlx::query(
        "SELECT completion_policy_id, revision_number, required_evidence_kinds \
         FROM completion_policy_revision \
         WHERE tenant_id = $1 AND completion_policy_revision_id = $2",
    )
    .bind(tenant_id)
    .bind(request.completion_policy_revision_id)
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or(ApiError::BadRequest(
        "completion policy revision is not in this tenant",
    ))?;
    let policy_id: Uuid = revision.try_get("completion_policy_id")?;
    let revision_number: i32 = revision.try_get("revision_number")?;
    let required_values: Value = revision.try_get("required_evidence_kinds")?;
    let required_names: Vec<String> = serde_json::from_value(required_values)
        .map_err(|_| ApiError::BadRequest("policy evidence kinds are invalid"))?;
    let required_evidence_kinds: BTreeSet<EvidenceKind> = required_names
        .iter()
        .map(|kind| parse_evidence_kind(kind))
        .collect::<Result<_, _>>()?;
    if required_evidence_kinds.len() != required_names.len() {
        return Err(ApiError::BadRequest("policy evidence kinds are duplicated"));
    }
    let policy_revision = CompletionPolicyRevision::new(
        tenant_id,
        policy_id,
        revision_number as u32,
        required_evidence_kinds,
    )
    .map_err(map_kernel_error)?;
    let evidence_rows = sqlx::query(
        "SELECT decision_evidence_reference_id, evidence_kind, source_authority, \
                source_snapshot_reference, source_digest, source_version, \
                assessment_result_status, observed_at \
         FROM decision_evidence_reference \
         WHERE tenant_id = $1 AND learner_id = $2 \
           AND learning_registration_id = $3 \
           AND decision_evidence_reference_id = ANY($4)",
    )
    .bind(tenant_id)
    .bind(learner_id)
    .bind(learning_registration_id)
    .bind(&request.evidence_reference_ids)
    .fetch_all(&mut *transaction)
    .await?;
    if evidence_rows.len() != request.evidence_reference_ids.len() {
        return Err(ApiError::BadRequest(
            "evidence reference is missing or foreign",
        ));
    }
    let mut evidence = Vec::with_capacity(evidence_rows.len());
    for row in evidence_rows {
        let evidence_id: Uuid = row.try_get("decision_evidence_reference_id")?;
        let evidence_kind_name: String = row.try_get("evidence_kind")?;
        let source_metadata = EvidenceSourceMetadata::new(
            row.try_get::<String, _>("source_authority")?,
            row.try_get::<String, _>("source_snapshot_reference")?,
            row.try_get::<String, _>("source_digest")?,
            row.try_get::<String, _>("source_version")?,
        )
        .map_err(map_kernel_error)?;
        let assessment_result_status_name: Option<String> =
            row.try_get("assessment_result_status")?;
        let assessment_result_status = assessment_result_status_name
            .as_deref()
            .map(parse_assessment_result_status)
            .transpose()?;
        evidence.push(
            DecisionEvidenceReference::from_existing_with_assessment_status(
                tenant_id,
                learner_id,
                evidence_id,
                parse_evidence_kind(&evidence_kind_name)?,
                source_metadata,
                row.try_get("observed_at")?,
                assessment_result_status,
            )
            .map_err(map_kernel_error)?,
        );
    }
    let decision = evaluate_completion(
        tenant_id,
        learner_id,
        policy_revision,
        &evidence,
        evaluated_at,
    )
    .map_err(map_kernel_error)?;
    let decision_row = sqlx::query(
        "INSERT INTO completion_decision \
         (completion_decision_id, tenant_id, learner_id, learning_registration_id, \
          completion_policy_revision_id, replay_fingerprint, evaluated_at, supersedes_decision_id) \
         SELECT $1, $2, $3, $4, $5, $6, $7, $8 \
         FROM learning_registration \
         WHERE tenant_id = $2 AND learner_id = $3 AND learning_registration_id = $4 \
           AND registration_status IN ('registered', 'launched', 'completed') \
         RETURNING completion_decision_id",
    )
    .bind(decision.decision_id)
    .bind(tenant_id)
    .bind(learner_id)
    .bind(learning_registration_id)
    .bind(request.completion_policy_revision_id)
    .bind(&decision.replay_fingerprint)
    .bind(evaluated_at)
    .bind(request.supersedes_decision_id)
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or(ApiError::BadRequest(
        "learning registration is not in this tenant",
    ))?;
    let completion_decision_id: Uuid = decision_row.try_get("completion_decision_id")?;
    for evidence_id in &request.evidence_reference_ids {
        sqlx::query(
            "INSERT INTO completion_decision_evidence \
             (completion_decision_id, decision_evidence_reference_id, tenant_id) \
             VALUES ($1, $2, $3)",
        )
        .bind(completion_decision_id)
        .bind(evidence_id)
        .bind(tenant_id)
        .execute(&mut *transaction)
        .await?;
    }
    sqlx::query(
        "UPDATE learning_registration SET registration_status = 'completed' \
         WHERE tenant_id = $1 AND learner_id = $2 AND learning_registration_id = $3",
    )
    .bind(tenant_id)
    .bind(learner_id)
    .bind(learning_registration_id)
    .execute(&mut *transaction)
    .await?;
    let source_version = format!("policy-revision/{revision_number}");
    let action_name = if request.supersedes_decision_id.is_some() {
        "completion_decision.corrected"
    } else {
        "completion_decision.published"
    };
    record_audit_event(
        &mut transaction,
        AuditEvent {
            tenant_id,
            correlation_id,
            action_name,
            entity_type: "completion_decision",
            entity_id: completion_decision_id,
            source_authority: "lms-policy-engine",
            source_version: &source_version,
            occurred_at: evaluated_at,
        },
    )
    .await?;
    transaction.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(CompletionDecisionResponse {
            completion_decision_id,
            tenant_id,
            learner_id,
            learning_registration_id,
            supersedes_decision_id: request.supersedes_decision_id,
            replay_fingerprint: decision.replay_fingerprint,
            evaluated_at,
            completion_status: "completed",
        }),
    ))
}

async fn create_credential(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((tenant_id, learner_id, learning_registration_id, completion_decision_id)): Path<(
        Uuid,
        Uuid,
        Uuid,
        Uuid,
    )>,
    Json(request): Json<CreateCredentialRequest>,
) -> Result<(StatusCode, Json<CredentialResponse>), ApiError> {
    if tenant_id.is_nil()
        || learner_id.is_nil()
        || learning_registration_id.is_nil()
        || completion_decision_id.is_nil()
        || request.credential_authority.trim().is_empty()
        || request.external_credential_reference.trim().is_empty()
    {
        return Err(ApiError::BadRequest("credential references are required"));
    }

    let correlation_id = request_correlation_id(&headers)?;
    let mut transaction = begin_tenant_transaction(&state.pool, tenant_id).await?;
    let credential = sqlx::query(
        "INSERT INTO credential_record \
         (tenant_id, learner_id, learning_registration_id, completion_decision_id, \
          credential_authority, external_credential_reference, credential_status) \
         SELECT decision.tenant_id, decision.learner_id, decision.learning_registration_id, \
                decision.completion_decision_id, $5, $6, 'issued' \
         FROM completion_decision AS decision \
         JOIN learning_registration AS registration \
           ON registration.tenant_id = decision.tenant_id \
          AND registration.learner_id = decision.learner_id \
          AND registration.learning_registration_id = decision.learning_registration_id \
         WHERE decision.tenant_id = $1 AND decision.learner_id = $2 \
           AND decision.learning_registration_id = $3 \
           AND decision.completion_decision_id = $4 \
           AND registration.registration_status = 'completed' \
         RETURNING credential_record_id, issued_at",
    )
    .bind(tenant_id)
    .bind(learner_id)
    .bind(learning_registration_id)
    .bind(completion_decision_id)
    .bind(&request.credential_authority)
    .bind(&request.external_credential_reference)
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or(ApiError::BadRequest(
        "completed learning registration and decision are required",
    ))?;
    let credential_record_id: Uuid = credential.try_get("credential_record_id")?;
    let issued_at: DateTime<Utc> = credential.try_get("issued_at")?;
    record_audit_event(
        &mut transaction,
        AuditEvent {
            tenant_id,
            correlation_id,
            action_name: "credential_record.issued",
            entity_type: "credential_record",
            entity_id: credential_record_id,
            source_authority: &request.credential_authority,
            source_version: "credential-reference/v1",
            occurred_at: issued_at,
        },
    )
    .await?;
    transaction.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(CredentialResponse {
            credential_record_id,
            tenant_id,
            learner_id,
            learning_registration_id,
            completion_decision_id,
            credential_authority: request.credential_authority,
            external_credential_reference: request.external_credential_reference,
            credential_status: "issued".to_owned(),
            issued_at,
            revoked_at: None,
        }),
    ))
}

async fn revoke_credential(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((
        tenant_id,
        learner_id,
        learning_registration_id,
        completion_decision_id,
        credential_record_id,
    )): Path<(Uuid, Uuid, Uuid, Uuid, Uuid)>,
) -> Result<(StatusCode, Json<CredentialResponse>), ApiError> {
    if tenant_id.is_nil()
        || learner_id.is_nil()
        || learning_registration_id.is_nil()
        || completion_decision_id.is_nil()
        || credential_record_id.is_nil()
    {
        return Err(ApiError::BadRequest("credential references are required"));
    }

    let correlation_id = request_correlation_id(&headers)?;
    let mut transaction = begin_tenant_transaction(&state.pool, tenant_id).await?;
    let credential = sqlx::query(
        "UPDATE credential_record SET credential_status = 'revoked', revoked_at = now() \
         WHERE tenant_id = $1 AND learner_id = $2 AND learning_registration_id = $3 \
           AND completion_decision_id = $4 AND credential_record_id = $5 \
           AND credential_status = 'issued' \
         RETURNING credential_record_id, tenant_id, learner_id, learning_registration_id, \
                   completion_decision_id, credential_authority, external_credential_reference, \
                   credential_status, issued_at, revoked_at",
    )
    .bind(tenant_id)
    .bind(learner_id)
    .bind(learning_registration_id)
    .bind(completion_decision_id)
    .bind(credential_record_id)
    .fetch_optional(&mut *transaction)
    .await?;
    let newly_revoked = credential.is_some();
    let credential = match credential {
        Some(credential) => credential,
        None => sqlx::query(
            "SELECT credential_record_id, tenant_id, learner_id, learning_registration_id, \
                    completion_decision_id, credential_authority, external_credential_reference, \
                    credential_status, issued_at, revoked_at \
             FROM credential_record \
             WHERE tenant_id = $1 AND learner_id = $2 AND learning_registration_id = $3 \
               AND completion_decision_id = $4 AND credential_record_id = $5",
        )
        .bind(tenant_id)
        .bind(learner_id)
        .bind(learning_registration_id)
        .bind(completion_decision_id)
        .bind(credential_record_id)
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(ApiError::BadRequest("credential is not in this tenant"))?,
    };
    let credential_record_id: Uuid = credential.try_get("credential_record_id")?;
    let credential_tenant_id: Uuid = credential.try_get("tenant_id")?;
    let credential_learner_id: Uuid = credential.try_get("learner_id")?;
    let credential_registration_id: Uuid = credential.try_get("learning_registration_id")?;
    let credential_decision_id: Uuid = credential.try_get("completion_decision_id")?;
    let credential_authority: String = credential.try_get("credential_authority")?;
    let external_credential_reference: String =
        credential.try_get("external_credential_reference")?;
    let credential_status: String = credential.try_get("credential_status")?;
    let issued_at: DateTime<Utc> = credential.try_get("issued_at")?;
    let revoked_at: Option<DateTime<Utc>> = credential.try_get("revoked_at")?;
    if newly_revoked && let Some(revoked_at) = revoked_at {
        record_audit_event(
            &mut transaction,
            AuditEvent {
                tenant_id,
                correlation_id,
                action_name: "credential_record.revoked",
                entity_type: "credential_record",
                entity_id: credential_record_id,
                source_authority: &credential_authority,
                source_version: "credential-reference/v1",
                occurred_at: revoked_at,
            },
        )
        .await?;
    }
    transaction.commit().await?;

    Ok((
        StatusCode::OK,
        Json(CredentialResponse {
            credential_record_id,
            tenant_id: credential_tenant_id,
            learner_id: credential_learner_id,
            learning_registration_id: credential_registration_id,
            completion_decision_id: credential_decision_id,
            credential_authority,
            external_credential_reference,
            credential_status,
            issued_at,
            revoked_at,
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
