\set ON_ERROR_STOP on

BEGIN;
SET LOCAL app.tenant_id = '00000000-0000-0000-0000-000000000101';

INSERT INTO learning_tenant (tenant_id, tenant_slug)
VALUES ('00000000-0000-0000-0000-000000000101', 'constraint-contract');

INSERT INTO login_identity_reference
    (login_identity_id, identity_authority, external_subject_reference)
VALUES
    ('00000000-0000-0000-0000-000000000201', 'keyverse', 'constraint-learner-a'),
    ('00000000-0000-0000-0000-000000000202', 'keyverse', 'constraint-learner-b');

INSERT INTO learner_profile (learner_id, login_identity_id)
VALUES
    ('00000000-0000-0000-0000-000000000301', '00000000-0000-0000-0000-000000000201'),
    ('00000000-0000-0000-0000-000000000302', '00000000-0000-0000-0000-000000000202');

INSERT INTO tenant_membership
    (tenant_id, learner_id, membership_status, valid_from)
VALUES
    ('00000000-0000-0000-0000-000000000101', '00000000-0000-0000-0000-000000000301', 'active', '2026-01-01T00:00:00Z'),
    ('00000000-0000-0000-0000-000000000101', '00000000-0000-0000-0000-000000000302', 'active', '2026-01-01T00:00:00Z');

-- Adjacent half-open ranges are valid.
INSERT INTO learning_affiliation
    (tenant_id, learner_id, affiliation_kind, valid_from, valid_to)
VALUES
    ('00000000-0000-0000-0000-000000000101', '00000000-0000-0000-0000-000000000301', 'partner', '2026-01-01T00:00:00Z', '2026-02-01T00:00:00Z'),
    ('00000000-0000-0000-0000-000000000101', '00000000-0000-0000-0000-000000000301', 'partner', '2026-02-01T00:00:00Z', '2026-03-01T00:00:00Z');

DO $contract$
BEGIN
    BEGIN
        INSERT INTO learning_affiliation
            (tenant_id, learner_id, affiliation_kind, valid_from, valid_to)
        VALUES
            ('00000000-0000-0000-0000-000000000101', '00000000-0000-0000-0000-000000000301', 'partner', '2026-01-15T00:00:00Z', '2026-02-15T00:00:00Z');
        RAISE EXCEPTION 'overlapping affiliation range was accepted';
    EXCEPTION
        WHEN exclusion_violation THEN NULL;
    END;

    BEGIN
        INSERT INTO learning_affiliation
            (tenant_id, learner_id, affiliation_kind, valid_from, valid_to)
        VALUES
            ('00000000-0000-0000-0000-000000000101', '00000000-0000-0000-0000-000000000301', 'customer', '2026-04-01T00:00:00Z', '2026-03-01T00:00:00Z');
        RAISE EXCEPTION 'reversed affiliation range was accepted';
    EXCEPTION
        WHEN check_violation THEN NULL;
    END;
END
$contract$;

INSERT INTO completion_policy
    (completion_policy_id, tenant_id, policy_name)
VALUES
    ('00000000-0000-0000-0000-000000000401', '00000000-0000-0000-0000-000000000101', 'constraint-policy');

INSERT INTO completion_policy_revision
    (completion_policy_revision_id, tenant_id, completion_policy_id, revision_number, required_evidence_kinds, published_at)
VALUES
    ('00000000-0000-0000-0000-000000000402', '00000000-0000-0000-0000-000000000101', '00000000-0000-0000-0000-000000000401', 1, '["activity"]', '2026-01-01T00:00:00Z');

INSERT INTO decision_evidence_reference
    (decision_evidence_reference_id, tenant_id, learner_id, evidence_kind, source_authority, source_snapshot_reference, source_digest, source_version, observed_at)
VALUES
    ('00000000-0000-0000-0000-000000000501', '00000000-0000-0000-0000-000000000101', '00000000-0000-0000-0000-000000000301', 'activity', 'learning_record_store', 'snapshot-a', 'digest-a', 'v1', '2026-01-01T00:00:00Z');

INSERT INTO course_offering
    (course_offering_id, tenant_id, offering_name, content_release_reference)
VALUES
    ('00000000-0000-0000-0000-000000000701', '00000000-0000-0000-0000-000000000101', 'constraint-offering-a', 'content-release/constraint-a'),
    ('00000000-0000-0000-0000-000000000702', '00000000-0000-0000-0000-000000000101', 'constraint-offering-b', 'content-release/constraint-b');

INSERT INTO access_entitlement
    (access_entitlement_id, tenant_id, learner_id, source_authority, external_entitlement_reference, source_digest, source_version, valid_from)
VALUES
    ('00000000-0000-0000-0000-000000000711', '00000000-0000-0000-0000-000000000101', '00000000-0000-0000-0000-000000000301', 'constraint-authority', 'entitlement-a', 'digest-a', 'v1', '2026-01-01T00:00:00Z'),
    ('00000000-0000-0000-0000-000000000712', '00000000-0000-0000-0000-000000000101', '00000000-0000-0000-0000-000000000302', 'constraint-authority', 'entitlement-b', 'digest-b', 'v1', '2026-01-01T00:00:00Z');

INSERT INTO enrollment_record
    (enrollment_record_id, tenant_id, learner_id, course_offering_id, access_entitlement_id)
VALUES
    ('00000000-0000-0000-0000-000000000721', '00000000-0000-0000-0000-000000000101', '00000000-0000-0000-0000-000000000301', '00000000-0000-0000-0000-000000000701', '00000000-0000-0000-0000-000000000711'),
    ('00000000-0000-0000-0000-000000000722', '00000000-0000-0000-0000-000000000101', '00000000-0000-0000-0000-000000000302', '00000000-0000-0000-0000-000000000701', '00000000-0000-0000-0000-000000000712');

INSERT INTO learning_registration
    (learning_registration_id, tenant_id, learner_id, enrollment_record_id, external_registration_reference)
VALUES
    ('00000000-0000-0000-0000-000000000731', '00000000-0000-0000-0000-000000000101', '00000000-0000-0000-0000-000000000301', '00000000-0000-0000-0000-000000000721', 'constraint-registration-a'),
    ('00000000-0000-0000-0000-000000000732', '00000000-0000-0000-0000-000000000101', '00000000-0000-0000-0000-000000000302', '00000000-0000-0000-0000-000000000722', 'constraint-registration-b');

INSERT INTO completion_decision
    (completion_decision_id, tenant_id, learner_id, learning_registration_id, completion_policy_revision_id, replay_fingerprint, evaluated_at)
VALUES
    ('00000000-0000-0000-0000-000000000601', '00000000-0000-0000-0000-000000000101', '00000000-0000-0000-0000-000000000301', '00000000-0000-0000-0000-000000000731', '00000000-0000-0000-0000-000000000402', 'fingerprint-a', '2026-01-01T00:00:00Z'),
    ('00000000-0000-0000-0000-000000000602', '00000000-0000-0000-0000-000000000101', '00000000-0000-0000-0000-000000000302', '00000000-0000-0000-0000-000000000732', '00000000-0000-0000-0000-000000000402', 'fingerprint-b', '2026-01-01T00:00:00Z');

DO $contract$
BEGIN
    BEGIN
        INSERT INTO enrollment_record
            (tenant_id, learner_id, course_offering_id, access_entitlement_id)
        VALUES
            ('00000000-0000-0000-0000-000000000101', '00000000-0000-0000-0000-000000000302', '00000000-0000-0000-0000-000000000702', '00000000-0000-0000-0000-000000000711');
        RAISE EXCEPTION 'cross-learner entitlement enrollment was accepted';
    EXCEPTION
        WHEN foreign_key_violation THEN NULL;
    END;

    BEGIN
        INSERT INTO learning_registration
            (tenant_id, learner_id, enrollment_record_id, external_registration_reference)
        VALUES
            ('00000000-0000-0000-0000-000000000101', '00000000-0000-0000-0000-000000000302', '00000000-0000-0000-0000-000000000721', 'cross-learner-registration');
        RAISE EXCEPTION 'cross-learner enrollment registration was accepted';
    EXCEPTION
        WHEN foreign_key_violation THEN NULL;
    END;

    BEGIN
        INSERT INTO completion_decision
            (tenant_id, learner_id, learning_registration_id, completion_policy_revision_id, replay_fingerprint, evaluated_at)
        VALUES
            ('00000000-0000-0000-0000-000000000101', '00000000-0000-0000-0000-000000000302', '00000000-0000-0000-0000-000000000731', '00000000-0000-0000-0000-000000000402', 'cross-learner-registration', '2026-01-02T00:00:00Z');
        RAISE EXCEPTION 'cross-learner registration decision was accepted';
    EXCEPTION
        WHEN foreign_key_violation THEN NULL;
    END;

    BEGIN
        INSERT INTO completion_decision
            (tenant_id, learner_id, learning_registration_id, completion_policy_revision_id, replay_fingerprint, evaluated_at, supersedes_decision_id)
        VALUES
            ('00000000-0000-0000-0000-000000000101', '00000000-0000-0000-0000-000000000302', '00000000-0000-0000-0000-000000000732', '00000000-0000-0000-0000-000000000402', 'fingerprint-c', '2026-01-02T00:00:00Z', '00000000-0000-0000-0000-000000000601');
        RAISE EXCEPTION 'cross-learner superseding decision was accepted';
    EXCEPTION
        WHEN foreign_key_violation THEN NULL;
    END;

    BEGIN
        INSERT INTO completion_decision_evidence
            (completion_decision_id, decision_evidence_reference_id, tenant_id, learner_id)
        VALUES
            ('00000000-0000-0000-0000-000000000602', '00000000-0000-0000-0000-000000000501', '00000000-0000-0000-0000-000000000101', '00000000-0000-0000-0000-000000000302');
        RAISE EXCEPTION 'cross-learner decision evidence was accepted';
    EXCEPTION
        WHEN foreign_key_violation THEN NULL;
    END;
END
$contract$;

ROLLBACK;
