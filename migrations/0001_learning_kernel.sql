-- The migration stores references to external authority; it never copies source payloads.
CREATE EXTENSION IF NOT EXISTS btree_gist;
CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE learning_tenant (
    tenant_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_slug text NOT NULL UNIQUE,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE login_identity_reference (
    login_identity_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    identity_authority text NOT NULL,
    external_subject_reference text NOT NULL,
    CONSTRAINT login_identity_reference_authority_subject_unique
        UNIQUE (identity_authority, external_subject_reference)
);

CREATE TABLE learner_profile (
    learner_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    login_identity_id uuid NOT NULL UNIQUE REFERENCES login_identity_reference (login_identity_id),
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE tenant_membership (
    tenant_membership_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES learning_tenant (tenant_id),
    learner_id uuid NOT NULL REFERENCES learner_profile (learner_id),
    membership_status text NOT NULL CHECK (membership_status IN ('active', 'suspended', 'ended')),
    valid_from timestamptz NOT NULL,
    valid_to timestamptz,
    CONSTRAINT tenant_membership_tenant_learner_unique UNIQUE (tenant_id, learner_id),
    CONSTRAINT tenant_membership_validity_check CHECK (valid_to IS NULL OR valid_to > valid_from)
);

CREATE TABLE learning_affiliation (
    learning_affiliation_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    learner_id uuid NOT NULL,
    affiliation_kind text NOT NULL,
    valid_from timestamptz NOT NULL,
    valid_to timestamptz,
    validity_range tstzrange GENERATED ALWAYS AS (
        tstzrange(valid_from, COALESCE(valid_to, 'infinity'::timestamptz), '[)')
    ) STORED,
    CONSTRAINT learning_affiliation_membership_fk
        FOREIGN KEY (tenant_id, learner_id)
        REFERENCES tenant_membership (tenant_id, learner_id),
    CONSTRAINT learning_affiliation_validity_check CHECK (valid_to IS NULL OR valid_to > valid_from),
    CONSTRAINT learning_affiliation_non_overlapping
        EXCLUDE USING gist (tenant_id WITH =, learner_id WITH =, affiliation_kind WITH =, validity_range WITH &&)
);

CREATE TABLE completion_policy (
    completion_policy_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES learning_tenant (tenant_id),
    policy_name text NOT NULL,
    CONSTRAINT completion_policy_tenant_name_unique UNIQUE (tenant_id, policy_name),
    CONSTRAINT completion_policy_tenant_id_unique UNIQUE (tenant_id, completion_policy_id)
);

CREATE TABLE completion_policy_revision (
    completion_policy_revision_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    completion_policy_id uuid NOT NULL,
    revision_number integer NOT NULL CHECK (revision_number > 0),
    required_evidence_kinds jsonb NOT NULL,
    published_at timestamptz NOT NULL,
    CONSTRAINT completion_policy_revision_policy_fk
        FOREIGN KEY (tenant_id, completion_policy_id)
        REFERENCES completion_policy (tenant_id, completion_policy_id),
    CONSTRAINT completion_policy_revision_unique UNIQUE (tenant_id, completion_policy_id, revision_number),
    CONSTRAINT completion_policy_revision_requirements_check
        CHECK (jsonb_typeof(required_evidence_kinds) = 'array' AND jsonb_array_length(required_evidence_kinds) > 0),
    CONSTRAINT completion_policy_revision_identity_unique UNIQUE (tenant_id, completion_policy_revision_id)
);

CREATE TABLE decision_evidence_reference (
    decision_evidence_reference_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    learner_id uuid NOT NULL,
    evidence_kind text NOT NULL,
    source_authority text NOT NULL,
    source_snapshot_reference text NOT NULL,
    source_digest text NOT NULL,
    source_version text NOT NULL,
    observed_at timestamptz NOT NULL,
    CONSTRAINT decision_evidence_reference_membership_fk
        FOREIGN KEY (tenant_id, learner_id)
        REFERENCES tenant_membership (tenant_id, learner_id),
    CONSTRAINT decision_evidence_reference_identity_unique UNIQUE (tenant_id, decision_evidence_reference_id)
);

CREATE TABLE completion_decision (
    completion_decision_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    learner_id uuid NOT NULL,
    learning_registration_id uuid NOT NULL,
    completion_policy_revision_id uuid NOT NULL,
    replay_fingerprint text NOT NULL,
    evaluated_at timestamptz NOT NULL,
    supersedes_decision_id uuid,
    published_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT completion_decision_membership_fk
        FOREIGN KEY (tenant_id, learner_id)
        REFERENCES tenant_membership (tenant_id, learner_id),
    CONSTRAINT completion_decision_policy_revision_fk
        FOREIGN KEY (tenant_id, completion_policy_revision_id)
        REFERENCES completion_policy_revision (tenant_id, completion_policy_revision_id),
    CONSTRAINT completion_decision_supersedes_fk
        FOREIGN KEY (tenant_id, supersedes_decision_id)
        REFERENCES completion_decision (tenant_id, completion_decision_id),
    CONSTRAINT completion_decision_identity_unique UNIQUE (tenant_id, completion_decision_id)
);

CREATE TABLE completion_decision_evidence (
    completion_decision_id uuid NOT NULL,
    decision_evidence_reference_id uuid NOT NULL,
    tenant_id uuid NOT NULL,
    PRIMARY KEY (completion_decision_id, decision_evidence_reference_id),
    CONSTRAINT completion_decision_evidence_decision_fk
        FOREIGN KEY (tenant_id, completion_decision_id)
        REFERENCES completion_decision (tenant_id, completion_decision_id),
    CONSTRAINT completion_decision_evidence_reference_fk
        FOREIGN KEY (tenant_id, decision_evidence_reference_id)
        REFERENCES decision_evidence_reference (tenant_id, decision_evidence_reference_id)
);

CREATE TABLE course_offering (
    course_offering_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL REFERENCES learning_tenant (tenant_id),
    offering_name text NOT NULL,
    content_release_reference text NOT NULL,
    offering_status text NOT NULL DEFAULT 'active'
        CHECK (offering_status IN ('active', 'retired')),
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT course_offering_tenant_name_unique UNIQUE (tenant_id, offering_name),
    CONSTRAINT course_offering_identity_unique UNIQUE (tenant_id, course_offering_id)
);

CREATE TABLE access_entitlement (
    access_entitlement_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    learner_id uuid NOT NULL,
    source_authority text NOT NULL,
    external_entitlement_reference text NOT NULL,
    source_digest text NOT NULL,
    source_version text NOT NULL,
    valid_from timestamptz NOT NULL,
    valid_to timestamptz,
    CONSTRAINT access_entitlement_membership_fk
        FOREIGN KEY (tenant_id, learner_id)
        REFERENCES tenant_membership (tenant_id, learner_id),
    CONSTRAINT access_entitlement_validity_check
        CHECK (valid_to IS NULL OR valid_to > valid_from),
    CONSTRAINT access_entitlement_source_unique
        UNIQUE (tenant_id, source_authority, external_entitlement_reference),
    CONSTRAINT access_entitlement_identity_unique UNIQUE (tenant_id, access_entitlement_id)
);

CREATE TABLE enrollment_record (
    enrollment_record_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    learner_id uuid NOT NULL,
    course_offering_id uuid NOT NULL,
    access_entitlement_id uuid NOT NULL,
    enrollment_status text NOT NULL DEFAULT 'enrolled'
        CHECK (enrollment_status IN ('enrolled', 'completed', 'cancelled')),
    enrolled_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT enrollment_record_membership_fk
        FOREIGN KEY (tenant_id, learner_id)
        REFERENCES tenant_membership (tenant_id, learner_id),
    CONSTRAINT enrollment_record_offering_fk
        FOREIGN KEY (tenant_id, course_offering_id)
        REFERENCES course_offering (tenant_id, course_offering_id),
    CONSTRAINT enrollment_record_entitlement_fk
        FOREIGN KEY (tenant_id, access_entitlement_id)
        REFERENCES access_entitlement (tenant_id, access_entitlement_id),
    CONSTRAINT enrollment_record_learner_offering_unique
        UNIQUE (tenant_id, learner_id, course_offering_id),
    CONSTRAINT enrollment_record_identity_unique UNIQUE (tenant_id, enrollment_record_id)
);

CREATE TABLE learning_registration (
    learning_registration_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    learner_id uuid NOT NULL,
    enrollment_record_id uuid NOT NULL,
    external_registration_reference text NOT NULL,
    registration_status text NOT NULL DEFAULT 'registered'
        CHECK (registration_status IN ('registered', 'launched', 'completed', 'closed')),
    registered_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT learning_registration_membership_fk
        FOREIGN KEY (tenant_id, learner_id)
        REFERENCES tenant_membership (tenant_id, learner_id),
    CONSTRAINT learning_registration_enrollment_fk
        FOREIGN KEY (tenant_id, enrollment_record_id)
        REFERENCES enrollment_record (tenant_id, enrollment_record_id),
    CONSTRAINT learning_registration_enrollment_unique UNIQUE (tenant_id, enrollment_record_id),
    CONSTRAINT learning_registration_learner_identity_unique
        UNIQUE (tenant_id, learner_id, learning_registration_id),
    CONSTRAINT learning_registration_identity_unique UNIQUE (tenant_id, learning_registration_id)
);

ALTER TABLE completion_decision
    ADD CONSTRAINT completion_decision_registration_fk
    FOREIGN KEY (tenant_id, learning_registration_id)
    REFERENCES learning_registration (tenant_id, learning_registration_id);

CREATE TABLE learning_attempt (
    learning_attempt_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    learner_id uuid NOT NULL,
    learning_registration_id uuid NOT NULL,
    external_attempt_reference text NOT NULL,
    content_release_reference text NOT NULL,
    attempt_status text NOT NULL DEFAULT 'launched'
        CHECK (attempt_status IN ('launched', 'active', 'completed', 'abandoned')),
    launched_at timestamptz NOT NULL DEFAULT now(),
    closed_at timestamptz,
    CONSTRAINT learning_attempt_membership_fk
        FOREIGN KEY (tenant_id, learner_id)
        REFERENCES tenant_membership (tenant_id, learner_id),
    CONSTRAINT learning_attempt_registration_fk
        FOREIGN KEY (tenant_id, learner_id, learning_registration_id)
        REFERENCES learning_registration (tenant_id, learner_id, learning_registration_id),
    CONSTRAINT learning_attempt_reference_check
        CHECK (length(btrim(external_attempt_reference)) > 0
            AND length(btrim(content_release_reference)) > 0),
    CONSTRAINT learning_attempt_closed_time_check
        CHECK (closed_at IS NULL OR closed_at >= launched_at),
    CONSTRAINT learning_attempt_registration_reference_unique
        UNIQUE (tenant_id, learning_registration_id, external_attempt_reference),
    CONSTRAINT learning_attempt_learner_identity_unique
        UNIQUE (tenant_id, learner_id, learning_attempt_id),
    CONSTRAINT learning_attempt_identity_unique UNIQUE (tenant_id, learning_attempt_id)
);

CREATE TABLE progress_projection (
    progress_projection_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    learner_id uuid NOT NULL,
    learning_attempt_id uuid NOT NULL,
    source_authority text NOT NULL,
    external_activity_reference text NOT NULL,
    source_version text NOT NULL,
    source_digest text NOT NULL,
    progress_state text NOT NULL
        CHECK (progress_state IN ('not_started', 'in_progress', 'completed')),
    progress_percent double precision NOT NULL CHECK (progress_percent >= 0 AND progress_percent <= 100),
    observed_at timestamptz NOT NULL,
    recorded_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT progress_projection_attempt_fk
        FOREIGN KEY (tenant_id, learner_id, learning_attempt_id)
        REFERENCES learning_attempt (tenant_id, learner_id, learning_attempt_id),
    CONSTRAINT progress_projection_source_check
        CHECK (length(btrim(source_authority)) > 0
            AND length(btrim(external_activity_reference)) > 0
            AND length(btrim(source_version)) > 0
            AND length(btrim(source_digest)) > 0),
    CONSTRAINT progress_projection_source_unique
        UNIQUE (tenant_id, learning_attempt_id, source_authority,
                external_activity_reference, source_version),
    CONSTRAINT progress_projection_identity_unique UNIQUE (tenant_id, progress_projection_id)
);

ALTER TABLE learning_tenant ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenant_membership ENABLE ROW LEVEL SECURITY;
ALTER TABLE learning_affiliation ENABLE ROW LEVEL SECURITY;
ALTER TABLE completion_policy ENABLE ROW LEVEL SECURITY;
ALTER TABLE completion_policy_revision ENABLE ROW LEVEL SECURITY;
ALTER TABLE decision_evidence_reference ENABLE ROW LEVEL SECURITY;
ALTER TABLE completion_decision ENABLE ROW LEVEL SECURITY;
ALTER TABLE completion_decision_evidence ENABLE ROW LEVEL SECURITY;
ALTER TABLE course_offering ENABLE ROW LEVEL SECURITY;
ALTER TABLE access_entitlement ENABLE ROW LEVEL SECURITY;
ALTER TABLE enrollment_record ENABLE ROW LEVEL SECURITY;
ALTER TABLE learning_registration ENABLE ROW LEVEL SECURITY;
ALTER TABLE learning_attempt ENABLE ROW LEVEL SECURITY;
ALTER TABLE progress_projection ENABLE ROW LEVEL SECURITY;

CREATE POLICY learning_tenant_tenant_policy ON learning_tenant
    USING (tenant_id::text = current_setting('app.tenant_id', true));
CREATE POLICY tenant_membership_tenant_policy ON tenant_membership
    USING (tenant_id::text = current_setting('app.tenant_id', true));
CREATE POLICY learning_affiliation_tenant_policy ON learning_affiliation
    USING (tenant_id::text = current_setting('app.tenant_id', true));
CREATE POLICY completion_policy_tenant_policy ON completion_policy
    USING (tenant_id::text = current_setting('app.tenant_id', true));
CREATE POLICY completion_policy_revision_tenant_policy ON completion_policy_revision
    USING (tenant_id::text = current_setting('app.tenant_id', true));
CREATE POLICY decision_evidence_reference_tenant_policy ON decision_evidence_reference
    USING (tenant_id::text = current_setting('app.tenant_id', true));
CREATE POLICY completion_decision_tenant_policy ON completion_decision
    USING (tenant_id::text = current_setting('app.tenant_id', true));
CREATE POLICY completion_decision_evidence_tenant_policy ON completion_decision_evidence
    USING (tenant_id::text = current_setting('app.tenant_id', true));
CREATE POLICY course_offering_tenant_policy ON course_offering
    USING (tenant_id::text = current_setting('app.tenant_id', true));
CREATE POLICY access_entitlement_tenant_policy ON access_entitlement
    USING (tenant_id::text = current_setting('app.tenant_id', true));
CREATE POLICY enrollment_record_tenant_policy ON enrollment_record
    USING (tenant_id::text = current_setting('app.tenant_id', true));
CREATE POLICY learning_registration_tenant_policy ON learning_registration
    USING (tenant_id::text = current_setting('app.tenant_id', true));
CREATE POLICY learning_attempt_tenant_policy ON learning_attempt
    USING (tenant_id::text = current_setting('app.tenant_id', true));
CREATE POLICY progress_projection_tenant_policy ON progress_projection
    USING (tenant_id::text = current_setting('app.tenant_id', true));

-- The owning migration role must not be used by the application connection.
ALTER TABLE learning_tenant FORCE ROW LEVEL SECURITY;
ALTER TABLE tenant_membership FORCE ROW LEVEL SECURITY;
ALTER TABLE learning_affiliation FORCE ROW LEVEL SECURITY;
ALTER TABLE completion_policy FORCE ROW LEVEL SECURITY;
ALTER TABLE completion_policy_revision FORCE ROW LEVEL SECURITY;
ALTER TABLE decision_evidence_reference FORCE ROW LEVEL SECURITY;
ALTER TABLE completion_decision FORCE ROW LEVEL SECURITY;
ALTER TABLE completion_decision_evidence FORCE ROW LEVEL SECURITY;
ALTER TABLE course_offering FORCE ROW LEVEL SECURITY;
ALTER TABLE access_entitlement FORCE ROW LEVEL SECURITY;
ALTER TABLE enrollment_record FORCE ROW LEVEL SECURITY;
ALTER TABLE learning_registration FORCE ROW LEVEL SECURITY;
ALTER TABLE learning_attempt FORCE ROW LEVEL SECURITY;
ALTER TABLE progress_projection FORCE ROW LEVEL SECURITY;
