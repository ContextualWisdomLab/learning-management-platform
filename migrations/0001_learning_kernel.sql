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

ALTER TABLE learning_tenant ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenant_membership ENABLE ROW LEVEL SECURITY;
ALTER TABLE learning_affiliation ENABLE ROW LEVEL SECURITY;
ALTER TABLE completion_policy ENABLE ROW LEVEL SECURITY;
ALTER TABLE completion_policy_revision ENABLE ROW LEVEL SECURITY;
ALTER TABLE decision_evidence_reference ENABLE ROW LEVEL SECURITY;
ALTER TABLE completion_decision ENABLE ROW LEVEL SECURITY;
ALTER TABLE completion_decision_evidence ENABLE ROW LEVEL SECURITY;

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

-- The owning migration role must not be used by the application connection.
ALTER TABLE learning_tenant FORCE ROW LEVEL SECURITY;
ALTER TABLE tenant_membership FORCE ROW LEVEL SECURITY;
ALTER TABLE learning_affiliation FORCE ROW LEVEL SECURITY;
ALTER TABLE completion_policy FORCE ROW LEVEL SECURITY;
ALTER TABLE completion_policy_revision FORCE ROW LEVEL SECURITY;
ALTER TABLE decision_evidence_reference FORCE ROW LEVEL SECURITY;
ALTER TABLE completion_decision FORCE ROW LEVEL SECURITY;
ALTER TABLE completion_decision_evidence FORCE ROW LEVEL SECURITY;
