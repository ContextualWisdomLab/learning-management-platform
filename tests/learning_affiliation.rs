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
    assert_eq!(affiliation.affiliation_id(), "affiliation_external_learner");
    assert_eq!(affiliation.learner_id(), "learner_external");
    assert_eq!(affiliation.affiliation_kind(), AffiliationKind::Customer);
    assert_eq!(affiliation.orgmetra_worker_reference(), None);
    assert_eq!(affiliation.effective_from(), 1_798_761_600);
    assert_eq!(affiliation.effective_until(), None);
}

#[test]
fn partner_learner_does_not_require_an_orgmetra_worker_reference() {
    let affiliation = LearningAffiliation::new(
        "tenant_partner",
        "affiliation_partner",
        "learner_partner",
        AffiliationKind::Partner,
        None,
        1_798_761_600,
        None,
    )
    .expect("partner learners must not depend on Orgmetra");

    assert_eq!(affiliation.affiliation_kind(), AffiliationKind::Partner);
    assert_eq!(affiliation.orgmetra_worker_reference(), None);
}

#[test]
fn every_declared_non_employee_relationship_is_orgmetra_independent() {
    for (affiliation_kind, affiliation_id, learner_id) in [
        (
            AffiliationKind::Contractor,
            "affiliation_contractor",
            "learner_contractor",
        ),
        (
            AffiliationKind::Candidate,
            "affiliation_candidate",
            "learner_candidate",
        ),
        (
            AffiliationKind::Student,
            "affiliation_student",
            "learner_student",
        ),
        (
            AffiliationKind::Guardian,
            "affiliation_guardian",
            "learner_guardian",
        ),
        (
            AffiliationKind::AssociationMember,
            "affiliation_association_member",
            "learner_association_member",
        ),
        (
            AffiliationKind::PublicLearner,
            "affiliation_public_learner",
            "learner_public",
        ),
        (
            AffiliationKind::SelfSponsoredLearner,
            "affiliation_self_sponsored_learner",
            "learner_self_sponsored",
        ),
    ] {
        let affiliation = LearningAffiliation::new(
            "tenant_learning_network",
            affiliation_id,
            learner_id,
            affiliation_kind,
            None,
            1_798_761_600,
            None,
        )
        .expect("non-employee relationships must not depend on Orgmetra");

        assert_eq!(affiliation.affiliation_kind(), affiliation_kind);
        assert_eq!(affiliation.orgmetra_worker_reference(), None);
    }
}

#[test]
fn every_declared_non_employee_relationship_rejects_a_worker_reference() {
    for affiliation_kind in [
        AffiliationKind::Contractor,
        AffiliationKind::Candidate,
        AffiliationKind::Student,
        AffiliationKind::Guardian,
        AffiliationKind::AssociationMember,
        AffiliationKind::PublicLearner,
        AffiliationKind::SelfSponsoredLearner,
    ] {
        let error = LearningAffiliation::new(
            "tenant_learning_network",
            "affiliation_external_relationship",
            "learner_external_relationship",
            affiliation_kind,
            Some("worker_should_not_cross_boundary"),
            1_798_761_600,
            None,
        )
        .expect_err("non-employee relationships reject workforce identity");

        assert_eq!(error, LearningAffiliationError::UnexpectedWorkerReference);
    }
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

    assert_eq!(error, LearningAffiliationError::UnexpectedWorkerReference);
}

#[test]
fn customer_affiliation_rejects_an_orgmetra_worker_reference() {
    let error = LearningAffiliation::new(
        "tenant_academy",
        "affiliation_customer",
        "learner_customer",
        AffiliationKind::Customer,
        Some("worker_should_not_cross_boundary"),
        1_798_761_600,
        None,
    )
    .expect_err("customer learners must not be coupled to workforce identity");

    assert_eq!(error, LearningAffiliationError::UnexpectedWorkerReference);
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
        ("tenant_academy", " ", "learner_external", "affiliation_id"),
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
