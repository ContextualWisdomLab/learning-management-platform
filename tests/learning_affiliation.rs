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

    assert_eq!(affiliation.tenant_id(), "tenant_academy");
    assert_eq!(
        affiliation.affiliation_id(),
        "affiliation_external_learner"
    );
    assert_eq!(affiliation.learner_id(), "learner_external");
    assert_eq!(affiliation.affiliation_kind(), AffiliationKind::Customer);
    assert_eq!(affiliation.orgmetra_worker_reference(), None);
    assert_eq!(affiliation.effective_from(), 1_798_761_600);
    assert_eq!(affiliation.effective_until(), None);
}

#[test]
fn employee_affiliation_preserves_its_acl_reference_and_validity_window() {
    let affiliation = LearningAffiliation::new(
        "tenant_employer",
        "affiliation_employee",
        "learner_employee",
        AffiliationKind::Employee,
        Some("worker_reference_01"),
        1_798_761_600,
        Some(1_801_353_600),
    )
    .expect("a bounded employee affiliation with its ACL reference is valid");

    assert_eq!(
        affiliation.orgmetra_worker_reference(),
        Some("worker_reference_01")
    );
    assert_eq!(affiliation.effective_until(), Some(1_801_353_600));
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
fn employee_affiliation_rejects_a_blank_worker_reference() {
    let error = LearningAffiliation::new(
        "tenant_employer",
        "affiliation_employee",
        "learner_employee",
        AffiliationKind::Employee,
        Some("  "),
        1_798_761_600,
        None,
    )
    .expect_err("a blank workforce reference is equivalent to a missing one");

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
    for (tenant_id, affiliation_id, learner_id, field_name) in [
        (
            " ",
            "affiliation_external_learner",
            "learner_external",
            "tenant_id",
        ),
        (
            "tenant_academy",
            " ",
            "learner_external",
            "affiliation_id",
        ),
        (
            "tenant_academy",
            "affiliation_external_learner",
            " ",
            "learner_id",
        ),
    ] {
        let error = LearningAffiliation::new(
            tenant_id,
            affiliation_id,
            learner_id,
            AffiliationKind::Customer,
            None,
            1_798_761_600,
            None,
        )
        .expect_err("blank semantic identifiers must fail closed");

        assert_eq!(
            error,
            LearningAffiliationError::BlankSemanticIdentifier(field_name)
        );
    }
}
