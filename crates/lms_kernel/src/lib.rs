//! Domain rules shared by the learning-management modules.
//!
#![warn(missing_docs)]

//! This crate deliberately contains no HTTP, database, or external-provider code.
//! Those adapters must call these rules and preserve the returned references.

use std::collections::{BTreeSet, HashSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

/// The learner affiliation types supported by the first buyer journey.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AffiliationKind {
    /// A learner linked to an external worker record.
    Employee,
    /// A non-employee worker with a contracted engagement.
    Contractor,
    /// A learner affiliated through a partner organization.
    Partner,
    /// A learner affiliated through a customer organization.
    Customer,
    /// A learner preparing for a certification or selection process.
    Candidate,
    /// A learner in an education program.
    Student,
    /// A guardian affiliation where the learner is represented by another person.
    Guardian,
    /// A learner affiliated through a membership association.
    AssociationMember,
    /// A public learner without an organizational affiliation.
    PublicLearner,
    /// A learner paying or sponsoring their own access.
    SelfSponsored,
}

/// A time-bounded learner affiliation within a tenant.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LearningAffiliation {
    /// The tenant that owns the affiliation.
    pub tenant_id: Uuid,
    /// The learner receiving the affiliation.
    pub learner_id: Uuid,
    /// The affiliation record identifier.
    pub affiliation_id: Uuid,
    /// The role represented by this affiliation.
    pub affiliation_kind: AffiliationKind,
    /// The inclusive start of the valid-time interval.
    pub valid_from: DateTime<Utc>,
    /// The exclusive end of the valid-time interval, if known.
    pub valid_to: Option<DateTime<Utc>>,
}

impl LearningAffiliation {
    /// Creates an affiliation and rejects an empty or inverted valid-time interval.
    pub fn new(
        tenant_id: Uuid,
        learner_id: Uuid,
        affiliation_kind: AffiliationKind,
        valid_from: DateTime<Utc>,
        valid_to: Option<DateTime<Utc>>,
    ) -> Result<Self, KernelError> {
        if tenant_id.is_nil() || learner_id.is_nil() {
            return Err(KernelError::NilIdentifier);
        }
        if valid_to.is_some_and(|end| end <= valid_from) {
            return Err(KernelError::InvalidValidityInterval);
        }
        Ok(Self {
            tenant_id,
            learner_id,
            affiliation_id: Uuid::new_v4(),
            affiliation_kind,
            valid_from,
            valid_to,
        })
    }
}

/// The evidence categories a completion policy may require.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    /// Evidence projected from observed learning activity.
    Activity,
    /// Evidence projected from an assessment result snapshot.
    Assessment,
    /// Evidence projected from attendance or session participation.
    Attendance,
    /// Evidence projected from an entitlement authority.
    Entitlement,
}

/// The source metadata required to reference evidence owned by another system.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EvidenceSourceMetadata {
    /// The owning system, such as an LRS or assessment service.
    pub source_authority: String,
    /// An opaque source snapshot identifier; the source payload is not copied.
    pub source_snapshot_reference: String,
    /// The digest supplied by the source or computed over an approved snapshot.
    pub source_digest: String,
    /// The source contract or observed version.
    pub source_version: String,
}

impl EvidenceSourceMetadata {
    /// Creates source metadata while requiring every immutable reference field.
    pub fn new(
        source_authority: impl Into<String>,
        source_snapshot_reference: impl Into<String>,
        source_digest: impl Into<String>,
        source_version: impl Into<String>,
    ) -> Result<Self, KernelError> {
        let metadata = Self {
            source_authority: source_authority.into(),
            source_snapshot_reference: source_snapshot_reference.into(),
            source_digest: source_digest.into(),
            source_version: source_version.into(),
        };
        if [
            &metadata.source_authority,
            &metadata.source_snapshot_reference,
            &metadata.source_digest,
            &metadata.source_version,
        ]
        .iter()
        .any(|value| value.trim().is_empty())
        {
            return Err(KernelError::MissingEvidenceMetadata);
        }
        Ok(metadata)
    }
}

/// An immutable reference to evidence owned by another system.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DecisionEvidenceReference {
    /// The tenant evaluating the evidence.
    pub tenant_id: Uuid,
    /// The learner to whom the evidence belongs.
    pub learner_id: Uuid,
    /// The local reference identifier.
    pub evidence_id: Uuid,
    /// The evidence category used by policy evaluation.
    pub evidence_kind: EvidenceKind,
    /// The immutable metadata from the owning system.
    pub source_metadata: EvidenceSourceMetadata,
    /// When the source observation was made.
    pub observed_at: DateTime<Utc>,
}

impl DecisionEvidenceReference {
    /// Creates an evidence reference while requiring all trust-boundary metadata.
    pub fn new(
        tenant_id: Uuid,
        learner_id: Uuid,
        evidence_kind: EvidenceKind,
        source_metadata: EvidenceSourceMetadata,
        observed_at: DateTime<Utc>,
    ) -> Result<Self, KernelError> {
        if tenant_id.is_nil() || learner_id.is_nil() {
            return Err(KernelError::NilIdentifier);
        }
        Ok(Self {
            tenant_id,
            learner_id,
            evidence_id: Uuid::new_v4(),
            evidence_kind,
            source_metadata,
            observed_at,
        })
    }
}

/// An immutable revision of a completion policy.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CompletionPolicyRevision {
    /// The tenant that owns the policy.
    pub tenant_id: Uuid,
    /// The stable policy identifier.
    pub policy_id: Uuid,
    /// The immutable revision number.
    pub revision_number: u32,
    /// The evidence categories required to complete.
    pub required_evidence_kinds: BTreeSet<EvidenceKind>,
}

impl CompletionPolicyRevision {
    /// Creates a revision with a stable policy identity and at least one requirement.
    pub fn new(
        tenant_id: Uuid,
        policy_id: Uuid,
        revision_number: u32,
        required_evidence_kinds: BTreeSet<EvidenceKind>,
    ) -> Result<Self, KernelError> {
        if tenant_id.is_nil() || policy_id.is_nil() {
            return Err(KernelError::NilIdentifier);
        }
        if revision_number == 0 || required_evidence_kinds.is_empty() {
            return Err(KernelError::InvalidPolicyRevision);
        }
        Ok(Self {
            tenant_id,
            policy_id,
            revision_number,
            required_evidence_kinds,
        })
    }
}

/// The failure modes of the domain rules.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum KernelError {
    /// A nil UUID crossed the domain boundary.
    #[error("tenant and learner identifiers must be non-nil")]
    NilIdentifier,
    /// A valid-time interval is empty or inverted.
    #[error("validity interval must end after it starts")]
    InvalidValidityInterval,
    /// A policy revision is incomplete.
    #[error("policy revision must have a positive revision number and at least one requirement")]
    InvalidPolicyRevision,
    /// Evidence lacks an immutable source reference.
    #[error("evidence source metadata is required")]
    MissingEvidenceMetadata,
    /// A policy or evidence row belongs to another tenant or learner.
    #[error("tenant and learner boundaries must match the decision")]
    BoundaryMismatch,
    /// The same evidence reference was supplied more than once.
    #[error("evidence references must be unique")]
    DuplicateEvidence,
    /// One or more policy requirements were not satisfied.
    #[error("completion requirements are not satisfied")]
    IncompleteEvidence,
}

/// An immutable completion decision produced by a policy revision.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CompletionDecision {
    /// The tenant that owns the decision.
    pub tenant_id: Uuid,
    /// The learner evaluated by the decision.
    pub learner_id: Uuid,
    /// The unique decision identifier.
    pub decision_id: Uuid,
    /// The exact policy revision used for evaluation.
    pub policy_revision: CompletionPolicyRevision,
    /// The evidence references used by the decision, sorted by ID.
    pub evidence_ids: Vec<Uuid>,
    /// The decision observation time supplied by the caller.
    pub evaluated_at: DateTime<Utc>,
    /// A stable digest of the policy/evidence input for replay comparison.
    pub replay_fingerprint: String,
}

/// Evaluates a completion decision without reading an external payload.
///
/// The caller supplies an observation time so a replay can preserve the same
/// input. The random decision ID is intentionally separate from the stable
/// replay fingerprint.
pub fn evaluate_completion(
    tenant_id: Uuid,
    learner_id: Uuid,
    policy_revision: CompletionPolicyRevision,
    evidence: &[DecisionEvidenceReference],
    evaluated_at: DateTime<Utc>,
) -> Result<CompletionDecision, KernelError> {
    if tenant_id.is_nil() || learner_id.is_nil() || policy_revision.tenant_id != tenant_id {
        return Err(KernelError::BoundaryMismatch);
    }

    let mut evidence_ids = Vec::with_capacity(evidence.len());
    let mut evidence_kinds = BTreeSet::new();
    let mut seen_ids = HashSet::with_capacity(evidence.len());
    for reference in evidence {
        if reference.tenant_id != tenant_id || reference.learner_id != learner_id {
            return Err(KernelError::BoundaryMismatch);
        }
        if !seen_ids.insert(reference.evidence_id) {
            return Err(KernelError::DuplicateEvidence);
        }
        evidence_ids.push(reference.evidence_id);
        evidence_kinds.insert(reference.evidence_kind.clone());
    }
    if !policy_revision
        .required_evidence_kinds
        .is_subset(&evidence_kinds)
    {
        return Err(KernelError::IncompleteEvidence);
    }
    evidence_ids.sort_unstable();
    let replay_fingerprint = fingerprint(
        tenant_id,
        learner_id,
        &policy_revision,
        &evidence_ids,
        evaluated_at,
    );
    Ok(CompletionDecision {
        tenant_id,
        learner_id,
        decision_id: Uuid::new_v4(),
        policy_revision,
        evidence_ids,
        evaluated_at,
        replay_fingerprint,
    })
}

fn fingerprint(
    tenant_id: Uuid,
    learner_id: Uuid,
    policy_revision: &CompletionPolicyRevision,
    evidence_ids: &[Uuid],
    evaluated_at: DateTime<Utc>,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(tenant_id.as_bytes());
    hasher.update(learner_id.as_bytes());
    hasher.update(policy_revision.policy_id.as_bytes());
    hasher.update(policy_revision.revision_number.to_be_bytes());
    hasher.update(evaluated_at.to_rfc3339().as_bytes());
    for kind in &policy_revision.required_evidence_kinds {
        hasher.update(
            serde_json::to_string(kind)
                .expect("enum serialization cannot fail")
                .as_bytes(),
        );
    }
    for evidence_id in evidence_ids {
        hasher.update(evidence_id.as_bytes());
    }
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids() -> (Uuid, Uuid) {
        (Uuid::from_u128(1), Uuid::from_u128(2))
    }

    fn evidence(
        tenant_id: Uuid,
        learner_id: Uuid,
        kind: EvidenceKind,
    ) -> DecisionEvidenceReference {
        DecisionEvidenceReference::new(
            tenant_id,
            learner_id,
            kind,
            EvidenceSourceMetadata::new("learning_record_store", "snapshot-1", "digest-1", "v1")
                .expect("valid source metadata"),
            DateTime::from_timestamp(1_700_000_000, 0).expect("fixed timestamp"),
        )
        .expect("valid evidence")
    }

    #[test]
    fn supports_non_employee_affiliations_without_worker_records() {
        let (tenant_id, learner_id) = ids();
        let affiliation = LearningAffiliation::new(
            tenant_id,
            learner_id,
            AffiliationKind::Partner,
            DateTime::from_timestamp(1_700_000_000, 0).expect("fixed timestamp"),
            None,
        )
        .expect("valid affiliation");
        assert_eq!(affiliation.affiliation_kind, AffiliationKind::Partner);
        assert!(affiliation.valid_to.is_none());
    }

    #[test]
    fn rejects_inverted_affiliation_intervals() {
        let (tenant_id, learner_id) = ids();
        let start = DateTime::from_timestamp(1_700_000_001, 0).expect("fixed timestamp");
        let result = LearningAffiliation::new(
            tenant_id,
            learner_id,
            AffiliationKind::Employee,
            start,
            Some(start),
        );
        assert_eq!(result, Err(KernelError::InvalidValidityInterval));
    }

    #[test]
    fn completion_is_replay_equivalent_and_order_independent() {
        let (tenant_id, learner_id) = ids();
        let activity = evidence(tenant_id, learner_id, EvidenceKind::Activity);
        let assessment = evidence(tenant_id, learner_id, EvidenceKind::Assessment);
        let mut requirements = BTreeSet::new();
        requirements.insert(EvidenceKind::Activity);
        requirements.insert(EvidenceKind::Assessment);
        let policy = CompletionPolicyRevision::new(tenant_id, Uuid::from_u128(3), 1, requirements)
            .expect("valid policy");
        let at = DateTime::from_timestamp(1_700_000_002, 0).expect("fixed timestamp");
        let first = evaluate_completion(
            tenant_id,
            learner_id,
            policy.clone(),
            &[activity.clone(), assessment.clone()],
            at,
        )
        .expect("complete");
        let replay =
            evaluate_completion(tenant_id, learner_id, policy, &[assessment, activity], at)
                .expect("replay complete");
        assert_ne!(first.decision_id, replay.decision_id);
        assert_eq!(first.replay_fingerprint, replay.replay_fingerprint);
        assert_eq!(first.evidence_ids, replay.evidence_ids);
    }

    #[test]
    fn rejects_missing_and_cross_tenant_evidence() {
        let (tenant_id, learner_id) = ids();
        let policy = CompletionPolicyRevision::new(
            tenant_id,
            Uuid::from_u128(3),
            1,
            BTreeSet::from([EvidenceKind::Assessment]),
        )
        .expect("valid policy");
        let at = DateTime::from_timestamp(1_700_000_002, 0).expect("fixed timestamp");
        assert_eq!(
            evaluate_completion(tenant_id, learner_id, policy.clone(), &[], at),
            Err(KernelError::IncompleteEvidence)
        );
        let foreign = evidence(Uuid::from_u128(9), learner_id, EvidenceKind::Assessment);
        assert_eq!(
            evaluate_completion(tenant_id, learner_id, policy, &[foreign], at),
            Err(KernelError::BoundaryMismatch)
        );
    }
}
