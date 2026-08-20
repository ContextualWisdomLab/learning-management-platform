# UML and interaction baseline

This repository has no frontend surface yet, so there is no Figma file or
Storybook inventory for this slice. The diagrams describe the executable API
and persistence boundary; the first UI ADR must record its Figma file ID and
design-token/Storybook inventory.

## Completion correction sequence

```mermaid
sequenceDiagram
    participant Client
    participant API as lms_api
    participant DB as PostgreSQL/RLS
    participant Kernel as Rust kernel
    participant Audit as audit_event_record

    Client->>API: POST completion decision + optional supersedes_decision_id
    API->>DB: BEGIN; SET LOCAL app.tenant_id
    alt predecessor supplied
        API->>DB: verify same tenant, learner, registration
        DB-->>API: predecessor row or none
    end
    API->>DB: load policy revision and registration evidence
    API->>Kernel: evaluate_completion(policy, evidence, evaluated_at)
    Kernel-->>API: immutable decision + replay fingerprint
    API->>DB: INSERT new decision with predecessor relation
    API->>Audit: INSERT corrected or published provenance event
    API->>DB: COMMIT
    API-->>Client: decision response with supersedes_decision_id
```

## Domain and persistence relationships

```mermaid
classDiagram
    class LearningTenant {
      +UUID tenant_id
      +string tenant_slug
    }
    class LearnerProfile {
      +UUID learner_id
      +UUID tenant_id
    }
    class LearningRegistration {
      +UUID learning_registration_id
      +UUID tenant_id
      +UUID learner_id
    }
    class CompletionPolicyRevision {
      +UUID completion_policy_revision_id
      +int revision_number
    }
    class CompletionDecision {
      +UUID completion_decision_id
      +UUID supersedes_decision_id
      +string replay_fingerprint
      +timestamp evaluated_at
    }
    class AuditEventRecord {
      +UUID audit_event_record_id
      +UUID correlation_id
      +string action_name
      +string event_digest
    }

    LearningTenant "1" --> "many" LearnerProfile : owns
    LearnerProfile "1" --> "many" LearningRegistration : registers
    LearningRegistration "1" --> "many" CompletionDecision : has immutable history
    CompletionPolicyRevision "1" --> "many" CompletionDecision : evaluates
    CompletionDecision "0..1" --> "many" CompletionDecision : supersedes
    CompletionDecision "1" --> "many" AuditEventRecord : provenance
```

The self-relation is directional: the new decision points to the predecessor;
no update of the predecessor is permitted.
