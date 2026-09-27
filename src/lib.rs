//! Learning Management Platform domain kernel.
//!
//! This first slice owns only the time-varying link between a learner and a
//! tenant. Workforce identity remains behind the Orgmetra anti-corruption
//! boundary and is referenced only by employee affiliations.

/// The business relationship that gives a learner access to a tenant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AffiliationKind {
    /// A workforce learner whose worker identity is owned by Orgmetra.
    Employee,
    /// A learner receiving access as a customer of the tenant.
    Customer,
    /// A learner receiving access through a partner organization.
    Partner,
    /// A non-employee learner delivering services to the tenant.
    Contractor,
    /// A prospective learner participating before enrollment or employment.
    Candidate,
    /// A learner participating through an education relationship.
    Student,
    /// A learner participating as a guardian of another learner.
    Guardian,
    /// A learner participating through an association membership.
    AssociationMember,
    /// A learner using an offering open to the public.
    PublicLearner,
    /// A learner who sponsors their own access.
    SelfSponsoredLearner,
}

/// A time-varying learner-to-tenant relationship.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LearningAffiliation {
    tenant_id: String,
    affiliation_id: String,
    learner_id: String,
    affiliation_kind: AffiliationKind,
    orgmetra_worker_reference: Option<String>,
    effective_from: i64,
    effective_until: Option<i64>,
}

/// A fail-closed violation of a [`LearningAffiliation`] invariant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LearningAffiliationError {
    /// One of the repository-owned semantic identifiers was blank.
    BlankSemanticIdentifier(&'static str),
    /// An employee affiliation omitted its Orgmetra ACL reference.
    MissingEmployeeWorkerReference,
    /// A non-employee affiliation attempted to carry workforce identity.
    UnexpectedWorkerReference,
    /// The optional end instant did not occur after the start instant.
    InvalidValidityInterval,
}

impl LearningAffiliation {
    /// Constructs an affiliation after checking tenant, workforce, and time invariants.
    pub fn new(
        tenant_id: &str,
        affiliation_id: &str,
        learner_id: &str,
        affiliation_kind: AffiliationKind,
        orgmetra_worker_reference: Option<&str>,
        effective_from: i64,
        effective_until: Option<i64>,
    ) -> Result<Self, LearningAffiliationError> {
        for (field_name, identifier) in [
            ("tenant_id", tenant_id),
            ("affiliation_id", affiliation_id),
            ("learner_id", learner_id),
        ] {
            if identifier.trim().is_empty() {
                return Err(LearningAffiliationError::BlankSemanticIdentifier(
                    field_name,
                ));
            }
        }

        if effective_until.is_some_and(|end| end <= effective_from) {
            return Err(LearningAffiliationError::InvalidValidityInterval);
        }

        let worker_reference = match (affiliation_kind, orgmetra_worker_reference) {
            (AffiliationKind::Employee, Some(reference)) if !reference.trim().is_empty() => {
                Some(reference.to_owned())
            }
            (AffiliationKind::Employee, _) => {
                return Err(LearningAffiliationError::MissingEmployeeWorkerReference);
            }
            (_, Some(_)) => {
                return Err(LearningAffiliationError::UnexpectedWorkerReference);
            }
            (_, None) => None,
        };

        Ok(Self {
            tenant_id: tenant_id.to_owned(),
            affiliation_id: affiliation_id.to_owned(),
            learner_id: learner_id.to_owned(),
            affiliation_kind,
            orgmetra_worker_reference: worker_reference,
            effective_from,
            effective_until,
        })
    }

    /// Returns the tenant that grants this affiliation.
    #[must_use]
    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    /// Returns this affiliation aggregate's stable identifier.
    #[must_use]
    pub fn affiliation_id(&self) -> &str {
        &self.affiliation_id
    }

    /// Returns the learner linked to the tenant.
    #[must_use]
    pub fn learner_id(&self) -> &str {
        &self.learner_id
    }

    /// Returns the business relationship represented by this affiliation.
    #[must_use]
    pub const fn affiliation_kind(&self) -> AffiliationKind {
        self.affiliation_kind
    }

    /// Returns the workforce ACL reference, which exists only for employees.
    #[must_use]
    pub fn orgmetra_worker_reference(&self) -> Option<&str> {
        self.orgmetra_worker_reference.as_deref()
    }

    /// Returns the inclusive start instant as Unix epoch seconds.
    #[must_use]
    pub const fn effective_from(&self) -> i64 {
        self.effective_from
    }

    /// Returns the exclusive end instant as Unix epoch seconds, if bounded.
    #[must_use]
    pub const fn effective_until(&self) -> Option<i64> {
        self.effective_until
    }
}
