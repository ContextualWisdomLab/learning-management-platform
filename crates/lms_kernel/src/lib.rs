//! Domain rules shared by the learning-management modules.
//!
#![warn(missing_docs)]

//! This crate deliberately contains no HTTP, database, or external-provider code.
//! Those adapters must call these rules and preserve the returned references.

use std::collections::{BTreeSet, HashSet};

use chrono::{DateTime, SecondsFormat, Utc};
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
#[serde(try_from = "LearningAffiliationData")]
pub struct LearningAffiliation {
    /// The tenant that owns the affiliation.
    tenant_id: Uuid,
    /// The learner receiving the affiliation.
    learner_id: Uuid,
    /// The affiliation record identifier.
    affiliation_id: Uuid,
    /// The role represented by this affiliation.
    affiliation_kind: AffiliationKind,
    /// The opaque Orgmetra worker reference required only for employees.
    orgmetra_worker_reference: Option<String>,
    /// The inclusive start of the valid-time interval.
    valid_from: DateTime<Utc>,
    /// The exclusive end of the valid-time interval, if known.
    valid_to: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
struct LearningAffiliationData {
    tenant_id: Uuid,
    learner_id: Uuid,
    affiliation_id: Uuid,
    affiliation_kind: AffiliationKind,
    orgmetra_worker_reference: Option<String>,
    valid_from: DateTime<Utc>,
    valid_to: Option<DateTime<Utc>>,
}

impl LearningAffiliation {
    /// Creates an affiliation with a valid worker link and valid-time interval.
    pub fn new(
        tenant_id: Uuid,
        learner_id: Uuid,
        affiliation_kind: AffiliationKind,
        orgmetra_worker_reference: Option<String>,
        valid_from: DateTime<Utc>,
        valid_to: Option<DateTime<Utc>>,
    ) -> Result<Self, KernelError> {
        let affiliation_id = Uuid::new_v4();
        Self::validate(
            tenant_id,
            learner_id,
            affiliation_id,
            &affiliation_kind,
            orgmetra_worker_reference.as_deref(),
            valid_from,
            valid_to,
        )?;
        Ok(Self {
            tenant_id,
            learner_id,
            affiliation_id,
            affiliation_kind,
            orgmetra_worker_reference,
            valid_from,
            valid_to,
        })
    }

    fn validate(
        tenant_id: Uuid,
        learner_id: Uuid,
        affiliation_id: Uuid,
        affiliation_kind: &AffiliationKind,
        orgmetra_worker_reference: Option<&str>,
        valid_from: DateTime<Utc>,
        valid_to: Option<DateTime<Utc>>,
    ) -> Result<(), KernelError> {
        if tenant_id.is_nil() || learner_id.is_nil() || affiliation_id.is_nil() {
            return Err(KernelError::NilIdentifier);
        }
        if valid_to.is_some_and(|end| end <= valid_from) {
            return Err(KernelError::InvalidValidityInterval);
        }
        match (affiliation_kind, orgmetra_worker_reference) {
            (AffiliationKind::Employee, Some(worker_reference))
                if !worker_reference.trim().is_empty() => {}
            (AffiliationKind::Employee, _) => {
                return Err(KernelError::MissingOrgmetraWorkerReference);
            }
            (_, Some(_)) => return Err(KernelError::UnexpectedOrgmetraWorkerReference),
            (_, None) => {}
        }
        Ok(())
    }

    /// Returns the tenant that owns the affiliation.
    pub fn tenant_id(&self) -> Uuid {
        self.tenant_id
    }

    /// Returns the learner receiving the affiliation.
    pub fn learner_id(&self) -> Uuid {
        self.learner_id
    }

    /// Returns the affiliation record identifier.
    pub fn affiliation_id(&self) -> Uuid {
        self.affiliation_id
    }

    /// Returns the affiliation kind.
    pub fn affiliation_kind(&self) -> &AffiliationKind {
        &self.affiliation_kind
    }

    /// Returns the opaque Orgmetra worker reference, when applicable.
    pub fn orgmetra_worker_reference(&self) -> Option<&str> {
        self.orgmetra_worker_reference.as_deref()
    }

    /// Returns the inclusive start of the valid-time interval.
    pub fn valid_from(&self) -> DateTime<Utc> {
        self.valid_from
    }

    /// Returns the exclusive end of the valid-time interval, when known.
    pub fn valid_to(&self) -> Option<DateTime<Utc>> {
        self.valid_to
    }
}

impl TryFrom<LearningAffiliationData> for LearningAffiliation {
    type Error = KernelError;

    fn try_from(value: LearningAffiliationData) -> Result<Self, Self::Error> {
        Self::validate(
            value.tenant_id,
            value.learner_id,
            value.affiliation_id,
            &value.affiliation_kind,
            value.orgmetra_worker_reference.as_deref(),
            value.valid_from,
            value.valid_to,
        )?;
        Ok(Self {
            tenant_id: value.tenant_id,
            learner_id: value.learner_id,
            affiliation_id: value.affiliation_id,
            affiliation_kind: value.affiliation_kind,
            orgmetra_worker_reference: value.orgmetra_worker_reference,
            valid_from: value.valid_from,
            valid_to: value.valid_to,
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
#[serde(try_from = "CompletionPolicyRevisionData")]
pub struct CompletionPolicyRevision {
    /// The tenant that owns the policy.
    tenant_id: Uuid,
    /// The stable policy identifier.
    policy_id: Uuid,
    /// The immutable revision number.
    revision_number: u32,
    /// The evidence categories required to complete.
    required_evidence_kinds: BTreeSet<EvidenceKind>,
}

#[derive(Deserialize)]
struct CompletionPolicyRevisionData {
    tenant_id: Uuid,
    policy_id: Uuid,
    revision_number: u32,
    required_evidence_kinds: BTreeSet<EvidenceKind>,
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

    /// Returns the tenant that owns the policy.
    pub fn tenant_id(&self) -> Uuid {
        self.tenant_id
    }

    /// Returns the stable policy identifier.
    pub fn policy_id(&self) -> Uuid {
        self.policy_id
    }

    /// Returns the immutable revision number.
    pub fn revision_number(&self) -> u32 {
        self.revision_number
    }

    /// Returns the evidence categories required for completion.
    pub fn required_evidence_kinds(&self) -> &BTreeSet<EvidenceKind> {
        &self.required_evidence_kinds
    }
}

impl TryFrom<CompletionPolicyRevisionData> for CompletionPolicyRevision {
    type Error = KernelError;

    fn try_from(value: CompletionPolicyRevisionData) -> Result<Self, Self::Error> {
        Self::new(
            value.tenant_id,
            value.policy_id,
            value.revision_number,
            value.required_evidence_kinds,
        )
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
    /// An employee affiliation lacks its required Orgmetra worker reference.
    #[error("employee affiliation requires a nonblank Orgmetra worker reference")]
    MissingOrgmetraWorkerReference,
    /// A non-employee affiliation attempted to store an employee-only reference.
    #[error("Orgmetra worker reference is allowed only for employee affiliations")]
    UnexpectedOrgmetraWorkerReference,
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

    let mut evidence_references = Vec::with_capacity(evidence.len());
    let mut evidence_kinds = BTreeSet::new();
    let mut seen_ids = HashSet::with_capacity(evidence.len());
    for reference in evidence {
        if reference.tenant_id != tenant_id || reference.learner_id != learner_id {
            return Err(KernelError::BoundaryMismatch);
        }
        if !seen_ids.insert(reference.evidence_id) {
            return Err(KernelError::DuplicateEvidence);
        }
        evidence_references.push(reference);
        evidence_kinds.insert(reference.evidence_kind.clone());
    }
    if !policy_revision
        .required_evidence_kinds
        .is_subset(&evidence_kinds)
    {
        return Err(KernelError::IncompleteEvidence);
    }
    evidence_references.sort_unstable_by_key(|reference| reference.evidence_id);
    let evidence_ids = evidence_references
        .iter()
        .map(|reference| reference.evidence_id)
        .collect::<Vec<_>>();
    let replay_fingerprint = fingerprint(
        tenant_id,
        learner_id,
        &policy_revision,
        &evidence_references,
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
    evidence: &[&DecisionEvidenceReference],
    evaluated_at: DateTime<Utc>,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(tenant_id.as_bytes());
    hasher.update(learner_id.as_bytes());
    hasher.update(policy_revision.policy_id.as_bytes());
    hasher.update(policy_revision.revision_number.to_be_bytes());
    update_length_prefixed(
        &mut hasher,
        evaluated_at
            .to_rfc3339_opts(SecondsFormat::Nanos, true)
            .as_bytes(),
    );
    for kind in &policy_revision.required_evidence_kinds {
        let encoded = serde_json::to_string(kind).expect("enum serialization cannot fail");
        update_length_prefixed(&mut hasher, encoded.as_bytes());
    }
    for reference in evidence {
        hasher.update(reference.evidence_id.as_bytes());
        let encoded_kind = serde_json::to_string(&reference.evidence_kind)
            .expect("enum serialization cannot fail");
        update_length_prefixed(&mut hasher, encoded_kind.as_bytes());
        update_length_prefixed(
            &mut hasher,
            reference.source_metadata.source_authority.as_bytes(),
        );
        update_length_prefixed(
            &mut hasher,
            reference
                .source_metadata
                .source_snapshot_reference
                .as_bytes(),
        );
        update_length_prefixed(
            &mut hasher,
            reference.source_metadata.source_digest.as_bytes(),
        );
        update_length_prefixed(
            &mut hasher,
            reference.source_metadata.source_version.as_bytes(),
        );
        update_length_prefixed(
            &mut hasher,
            reference
                .observed_at
                .to_rfc3339_opts(SecondsFormat::Nanos, true)
                .as_bytes(),
        );
    }
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn update_length_prefixed(hasher: &mut Sha256, value: &[u8]) {
    hasher.update((value.len() as u64).to_be_bytes());
    hasher.update(value);
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
            None,
            DateTime::from_timestamp(1_700_000_000, 0).expect("fixed timestamp"),
            None,
        )
        .expect("valid affiliation");
        assert_eq!(affiliation.affiliation_kind, AffiliationKind::Partner);
        assert!(affiliation.valid_to.is_none());
    }

    #[test]
    fn supports_customer_self_sponsored_and_multi_tenant_affiliations() {
        let (_, learner_id) = ids();
        let at = DateTime::from_timestamp(1_700_000_000, 0).expect("fixed timestamp");
        let customer = LearningAffiliation::new(
            Uuid::from_u128(10),
            learner_id,
            AffiliationKind::Customer,
            None,
            at,
            None,
        )
        .expect("valid customer affiliation");
        let self_sponsored = LearningAffiliation::new(
            Uuid::from_u128(11),
            learner_id,
            AffiliationKind::SelfSponsored,
            None,
            at,
            None,
        )
        .expect("valid self-sponsored affiliation");
        assert_eq!(customer.affiliation_kind, AffiliationKind::Customer);
        assert_eq!(
            self_sponsored.affiliation_kind,
            AffiliationKind::SelfSponsored
        );
        assert_ne!(customer.tenant_id, self_sponsored.tenant_id);
    }

    #[test]
    fn employee_affiliation_requires_a_nonblank_orgmetra_worker_reference() {
        let (tenant_id, learner_id) = ids();
        let at = DateTime::from_timestamp(1_700_000_000, 0).expect("fixed timestamp");

        for worker_reference in [None, Some("   ".to_owned())] {
            assert_eq!(
                LearningAffiliation::new(
                    tenant_id,
                    learner_id,
                    AffiliationKind::Employee,
                    worker_reference,
                    at,
                    None,
                ),
                Err(KernelError::MissingOrgmetraWorkerReference)
            );
        }
    }

    #[test]
    fn employee_affiliation_preserves_its_orgmetra_worker_reference() {
        let (tenant_id, learner_id) = ids();
        let affiliation = LearningAffiliation::new(
            tenant_id,
            learner_id,
            AffiliationKind::Employee,
            Some("worker-42".to_owned()),
            DateTime::from_timestamp(1_700_000_000, 0).expect("fixed timestamp"),
            None,
        )
        .expect("employee affiliation with a worker reference");

        assert_eq!(
            affiliation.orgmetra_worker_reference.as_deref(),
            Some("worker-42")
        );
    }

    #[test]
    fn every_non_employee_affiliation_rejects_an_orgmetra_worker_reference() {
        let (tenant_id, learner_id) = ids();
        let at = DateTime::from_timestamp(1_700_000_000, 0).expect("fixed timestamp");
        let non_employee_kinds = [
            AffiliationKind::Contractor,
            AffiliationKind::Partner,
            AffiliationKind::Customer,
            AffiliationKind::Candidate,
            AffiliationKind::Student,
            AffiliationKind::Guardian,
            AffiliationKind::AssociationMember,
            AffiliationKind::PublicLearner,
            AffiliationKind::SelfSponsored,
        ];

        for affiliation_kind in non_employee_kinds {
            assert_eq!(
                LearningAffiliation::new(
                    tenant_id,
                    learner_id,
                    affiliation_kind,
                    Some("worker-42".to_owned()),
                    at,
                    None,
                ),
                Err(KernelError::UnexpectedOrgmetraWorkerReference)
            );
        }
    }

    #[test]
    fn rejects_inverted_affiliation_intervals() {
        let (tenant_id, learner_id) = ids();
        let start = DateTime::from_timestamp(1_700_000_001, 0).expect("fixed timestamp");
        let result = LearningAffiliation::new(
            tenant_id,
            learner_id,
            AffiliationKind::Employee,
            Some("worker-42".to_owned()),
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
    fn replay_fingerprint_changes_with_replay_relevant_evidence_metadata() {
        let (tenant_id, learner_id) = ids();
        let original = evidence(tenant_id, learner_id, EvidenceKind::Activity);
        let mut changed_digest = original.clone();
        changed_digest.source_metadata.source_digest = "digest-2".to_owned();
        let mut changed_version = original.clone();
        changed_version.source_metadata.source_version = "v2".to_owned();
        let mut changed_observation = original.clone();
        changed_observation.observed_at =
            DateTime::from_timestamp(1_700_000_001, 0).expect("fixed timestamp");
        let policy = CompletionPolicyRevision::new(
            tenant_id,
            Uuid::from_u128(3),
            1,
            BTreeSet::from([EvidenceKind::Activity]),
        )
        .expect("valid policy");
        let evaluated_at = DateTime::from_timestamp(1_700_000_002, 0).expect("fixed timestamp");

        let first = evaluate_completion(
            tenant_id,
            learner_id,
            policy.clone(),
            &[original],
            evaluated_at,
        )
        .expect("complete");
        for changed in [changed_digest, changed_version, changed_observation] {
            let changed = evaluate_completion(
                tenant_id,
                learner_id,
                policy.clone(),
                &[changed],
                evaluated_at,
            )
            .expect("complete");
            assert_ne!(first.replay_fingerprint, changed.replay_fingerprint);
        }
    }

    #[test]
    fn completion_rejects_blank_deserialized_evidence_metadata() {
        let (tenant_id, learner_id) = ids();
        let mut untrusted = evidence(tenant_id, learner_id, EvidenceKind::Activity);
        untrusted.source_metadata.source_digest = "   ".to_owned();
        let policy = CompletionPolicyRevision::new(
            tenant_id,
            Uuid::from_u128(3),
            1,
            BTreeSet::from([EvidenceKind::Activity]),
        )
        .expect("valid policy");
        let evaluated_at = DateTime::from_timestamp(1_700_000_002, 0).expect("fixed timestamp");

        assert_eq!(
            evaluate_completion(tenant_id, learner_id, policy, &[untrusted], evaluated_at),
            Err(KernelError::MissingEvidenceMetadata)
        );
    }

    #[test]
    fn deserialization_cannot_bypass_affiliation_and_policy_validation() {
        let invalid_affiliation = serde_json::json!({
            "tenant_id": Uuid::from_u128(1),
            "learner_id": Uuid::from_u128(2),
            "affiliation_id": Uuid::from_u128(3),
            "affiliation_kind": "employee",
            "orgmetra_worker_reference": null,
            "valid_from": "2023-11-14T22:13:20Z",
            "valid_to": null
        });
        assert!(serde_json::from_value::<LearningAffiliation>(invalid_affiliation).is_err());

        let invalid_policy = serde_json::json!({
            "tenant_id": Uuid::from_u128(1),
            "policy_id": Uuid::from_u128(3),
            "revision_number": 0,
            "required_evidence_kinds": []
        });
        assert!(serde_json::from_value::<CompletionPolicyRevision>(invalid_policy).is_err());

        let (tenant_id, learner_id) = ids();
        let affiliation = LearningAffiliation::new(
            tenant_id,
            learner_id,
            AffiliationKind::Employee,
            Some("worker-42".to_owned()),
            DateTime::from_timestamp(1_700_000_000, 0).expect("fixed timestamp"),
            None,
        )
        .expect("valid affiliation");
        let encoded = serde_json::to_value(&affiliation).expect("serialize affiliation");
        assert_eq!(
            serde_json::from_value::<LearningAffiliation>(encoded)
                .expect("deserialize affiliation"),
            affiliation
        );

        let policy = CompletionPolicyRevision::new(
            tenant_id,
            Uuid::from_u128(3),
            1,
            BTreeSet::from([EvidenceKind::Activity]),
        )
        .expect("valid policy");
        let encoded = serde_json::to_value(&policy).expect("serialize policy");
        assert_eq!(
            serde_json::from_value::<CompletionPolicyRevision>(encoded)
                .expect("deserialize policy"),
            policy
        );
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
