-- Correlation is the incident-trace lookup key for append-only audit events.
CREATE INDEX audit_event_record_tenant_correlation_idx
    ON audit_event_record (tenant_id, correlation_id, occurred_at, audit_event_record_id);
