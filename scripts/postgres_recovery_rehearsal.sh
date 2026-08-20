#!/usr/bin/env bash
set -euo pipefail

admin_db="${LMS_POSTGRES_ADMIN_DB:-postgres}"
run_stamp="$(date -u +%Y%m%d%H%M%S)_$$"
source_db="lms_recovery_source_${run_stamp}"
restore_db="lms_recovery_restore_${run_stamp}"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/lms-recovery.XXXXXX")"
dump_file="$work_dir/lms_kernel.dump"

psql_db() {
    local database_name="$1"
    shift
    psql --no-psqlrc --quiet --dbname="$database_name" --set=ON_ERROR_STOP=1 "$@"
}

cleanup() {
    dropdb --if-exists --force --maintenance-db="$admin_db" "$source_db" >/dev/null 2>&1 || true
    dropdb --if-exists --force --maintenance-db="$admin_db" "$restore_db" >/dev/null 2>&1 || true
    rm -f -- "$dump_file"
    rmdir -- "$work_dir" 2>/dev/null || true
}
trap cleanup EXIT

createdb --maintenance-db="$admin_db" "$source_db" >/dev/null
createdb --maintenance-db="$admin_db" "$restore_db" >/dev/null
psql_db "$source_db" --file migrations/0001_learning_kernel.sql >/dev/null

psql_db "$source_db" <<'SQL'
INSERT INTO learning_tenant (tenant_id, tenant_slug)
VALUES ('00000000-0000-4000-8000-000000000001', 'rehearsal-majority'),
       ('00000000-0000-4000-8000-000000000002', 'rehearsal-minority'),
       ('00000000-0000-4000-8000-000000000003', 'rehearsal-empty');

INSERT INTO audit_event_record (
    tenant_id, correlation_id, action_name, entity_type, entity_id,
    source_authority, source_version, event_digest, occurred_at
)
SELECT '00000000-0000-4000-8000-000000000001', gen_random_uuid(), 'recovery.rehearsal',
       'rehearsal', gen_random_uuid(), 'local-rehearsal', 'v1', md5(g::text),
       now() - make_interval(mins => g)
FROM generate_series(1, 900) AS series(g);

INSERT INTO audit_event_record (
    tenant_id, correlation_id, action_name, entity_type, entity_id,
    source_authority, source_version, event_digest, occurred_at
)
SELECT '00000000-0000-4000-8000-000000000002', gen_random_uuid(), 'recovery.rehearsal',
       'rehearsal', gen_random_uuid(), 'local-rehearsal', 'v1', md5(g::text),
       now() - make_interval(mins => g)
FROM generate_series(1, 100) AS series(g);
SQL

source_distribution="$(psql_db "$source_db" --tuples-only --no-align \
    --command "SELECT tenant_id, count(*) FROM audit_event_record GROUP BY tenant_id ORDER BY tenant_id;")"
source_total="$(psql_db "$source_db" --tuples-only --no-align \
    --command "SELECT count(*) FROM audit_event_record;" | tr -d '[:space:]')"
source_tenant_count="$(psql_db "$source_db" --tuples-only --no-align \
    --command "SELECT count(*) FROM audit_event_record GROUP BY tenant_id;" | wc -l | tr -d '[:space:]')"
source_skew="$(psql_db "$source_db" --tuples-only --no-align \
    --command "SELECT round(max(count)::numeric / min(count) FILTER (WHERE count > 0), 2) FROM (SELECT count(*) FROM audit_event_record GROUP BY tenant_id) AS counts;" \
    | tr -d '[:space:]')"
test "$source_total" = 1000
test "$source_tenant_count" = 2
test "$source_skew" = 9.00

backup_seconds="$(
    { /usr/bin/time -p pg_dump --format=custom --no-owner --file="$dump_file" "$source_db"; } 2>&1 \
        | awk '$1 == "real" { print $2 }'
)"
restore_seconds="$(
    { /usr/bin/time -p pg_restore --exit-on-error --no-owner --dbname="$restore_db" "$dump_file"; } 2>&1 \
        | awk '$1 == "real" { print $2 }'
)"

restore_distribution="$(psql_db "$restore_db" --tuples-only --no-align \
    --command "SELECT tenant_id, count(*) FROM audit_event_record GROUP BY tenant_id ORDER BY tenant_id;")"
restore_total="$(psql_db "$restore_db" --tuples-only --no-align \
    --command "SELECT count(*) FROM audit_event_record;" | tr -d '[:space:]')"
restore_tenant_count="$(psql_db "$restore_db" --tuples-only --no-align \
    --command "SELECT count(*) FROM audit_event_record GROUP BY tenant_id;" | wc -l | tr -d '[:space:]')"
test "$restore_distribution" = "$source_distribution"
test "$restore_total" = "$source_total"
test "$restore_tenant_count" = "$source_tenant_count"

policy_count="$(psql_db "$restore_db" --tuples-only --no-align \
    --command "SELECT count(*) FROM pg_policies WHERE schemaname = 'public';" | tr -d '[:space:]')"
forced_rls_count="$(psql_db "$restore_db" --tuples-only --no-align \
    --command "SELECT count(*) FROM pg_class AS relation JOIN pg_namespace AS namespace ON namespace.oid = relation.relnamespace WHERE namespace.nspname = 'public' AND relation.relkind = 'r' AND relation.relforcerowsecurity;" | tr -d '[:space:]')"
test "$policy_count" = 16
test "$forced_rls_count" = 16

psql_db "$restore_db" --file migrations/0002_audit_correlation_index.sql >/dev/null
forward_index_count="$(psql_db "$restore_db" --tuples-only --no-align \
    --command "SELECT count(*) FROM pg_indexes WHERE schemaname = 'public' AND indexname = 'audit_event_record_tenant_correlation_idx';" \
    | tr -d '[:space:]')"
test "$forward_index_count" = 1

printf 'backup_seconds %s\n' "$backup_seconds"
printf 'restore_seconds %s\n' "$restore_seconds"
printf 'dump_bytes %s\n' "$(wc -c < "$dump_file" | tr -d '[:space:]')"
printf 'audit_distribution\n%s\n' "$source_distribution"
printf 'audit_skew_ratio %s\n' "$source_skew"
printf 'restored_distribution\n%s\n' "$restore_distribution"
printf 'restored_policies %s\n' "$policy_count"
printf 'restored_forced_rls %s\n' "$forced_rls_count"
printf 'forward_index %s\n' "$forward_index_count"
printf 'postgres_recovery_rehearsal_passed\n'
