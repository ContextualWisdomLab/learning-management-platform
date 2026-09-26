use learning_management_platform::{
    AffiliationKind, LearningAffiliation, LearningAffiliationError,
};

#[test]
fn external_learner_does_not_require_an_orgmetra_worker_reference() {
    let affiliation = LearningAffiliation::new(
        "tenant_academy",
        "affiliation_external_learner",
        "learner_external",
        AffiliationKind::Customer,
        None,
        1_798_761_600,
        None,
    )
    .expect("external learners must not depend on Orgmetra");

    assert_eq!(affiliation.orgmetra_worker_reference(), None);
}

#[test]
fn employee_affiliation_requires_an_orgmetra_worker_reference() {
    let error = LearningAffiliation::new(
        "tenant_employer",
        "affiliation_employee",
        "learner_employee",
        AffiliationKind::Employee,
        None,
        1_798_761_600,
        None,
    )
    .expect_err("employee affiliations require the workforce ACL reference");

    assert_eq!(
        error,
        LearningAffiliationError::MissingEmployeeWorkerReference
    );
}

#[test]
fn non_employee_affiliation_rejects_an_orgmetra_worker_reference() {
    let error = LearningAffiliation::new(
        "tenant_partner",
        "affiliation_partner",
        "learner_partner",
        AffiliationKind::Partner,
        Some("worker_should_not_cross_boundary"),
        1_798_761_600,
        None,
    )
    .expect_err("non-employee learners must not be coupled to workforce identity");

    assert_eq!(
        error,
        LearningAffiliationError::UnexpectedWorkerReference
    );
}

#[test]
fn affiliation_validity_interval_must_move_forward() {
    let error = LearningAffiliation::new(
        "tenant_academy",
        "affiliation_expired",
        "learner_external",
        AffiliationKind::Customer,
        None,
        1_798_761_600,
        Some(1_798_761_600),
    )
    .expect_err("a closed interval would make replay ambiguous");

    assert_eq!(error, LearningAffiliationError::InvalidValidityInterval);
}

#[test]
fn semantic_identifiers_must_not_be_blank() {
    let error = LearningAffiliation::new(
        " ",
        "affiliation_external_learner",
        "learner_external",
        AffiliationKind::Customer,
        None,
        1_798_761_600,
        None,
    )
    .expect_err("blank tenant identity must fail closed");

    assert_eq!(
        error,
        LearningAffiliationError::BlankSemanticIdentifier("tenant_id")
    );
}
