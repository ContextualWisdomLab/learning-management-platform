# Operability

The repository currently publishes no service or release, opens no connection,
and has no runtime deployment. Its only operational surface is deterministic CI.

Before a service release, add structured audit events, readiness and liveness
probes, bounded retries, graceful connection closure, migration/rollback
runbooks, SLOs, and exact-revision build/SBOM/provenance evidence. Performance
claims, including page p95 at or below 20 ms, require a real deployable slice and
k6 evidence; no such claim is made here.

