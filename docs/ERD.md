# Proposed ERD

No migration is released. This logical model guides the next persistence slice
and does not authorize a database object yet.

```mermaid
erDiagram
    LEARNER ||--o{ LEARNING_AFFILIATION : holds
    TENANT ||--o{ LEARNING_AFFILIATION : grants
    LEARNER {
        text learner_id PK
    }
    TENANT {
        text tenant_id PK
    }
    LEARNING_AFFILIATION {
        text affiliation_id PK
        text tenant_id FK
        text learner_id FK
        text affiliation_kind
        text orgmetra_worker_reference
        timestamptz effective_from
        timestamptz effective_until
    }
```

The physical PostgreSQL design must preserve 3NF, tenant-scoped uniqueness, an
exclusive end instant, and the employee-only worker-reference invariant. It
requires migration and rollback tests before adoption.

