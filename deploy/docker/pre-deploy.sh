#!/usr/bin/env bash
# Database migrations, Elasticsearch indexes, and optional S3 bucket checks.
# Intended for Uncloud x-pre_deploy on artcraft-api (fail-fast).

set -euo pipefail

log() {
  echo "[pre-deploy] $*"
}

require_env() {
  local name="$1"
  if [ -z "${!name:-}" ]; then
    log "ERROR: required environment variable $name is not set"
    exit 1
  fi
}

run_mysql_migrations() {
  local mysql_host="${MYSQL_MIGRATION_HOST:-}"
  local mysql_port="${MYSQL_MIGRATION_PORT:-3306}"
  local mysql_user="${MYSQL_MIGRATION_USER:-}"
  local mysql_pass="${MYSQL_MIGRATION_PASSWORD:-}"
  local mysql_db="${MYSQL_MIGRATION_DATABASE:-storyteller}"

  if [ -z "$mysql_host" ] || [ -z "$mysql_user" ] || [ -z "$mysql_pass" ]; then
    log "ERROR: set MYSQL_MIGRATION_HOST, MYSQL_MIGRATION_USER, and MYSQL_MIGRATION_PASSWORD"
    exit 1
  fi

  export MYSQL_PWD="$mysql_pass"

  run_sql() {
    mysql -h "$mysql_host" -P "$mysql_port" -u "$mysql_user" "$mysql_db" "$@"
  }

  log "Applying SQL migrations from /opt/artcraft/migrations"
  run_sql -e "CREATE TABLE IF NOT EXISTS __diesel_schema_migrations (
    version VARCHAR(50) PRIMARY KEY NOT NULL,
    run_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP)"

  for dir in /opt/artcraft/migrations/*/; do
    [ -d "$dir" ] || continue
    name="$(basename "$dir")"
    version="$(echo "${name%%_*}" | tr -d '-')"
    if [ -n "$(run_sql -N -e "SELECT 1 FROM __diesel_schema_migrations WHERE version = '$version'")" ]; then
      continue
    fi
    log "Applying migration $name"
    run_sql < "$dir/up.sql"
    run_sql -e "INSERT INTO __diesel_schema_migrations (version) VALUES ('$version')"
  done

  log "MySQL migrations complete"
}

ensure_elasticsearch_indexes() {
  require_env ELASTICSEARCH_URL

  local defs_dir="/opt/artcraft/elasticsearch/index_definitions"
  if [ ! -d "$defs_dir" ]; then
    log "No Elasticsearch definitions at $defs_dir; skipping"
    return 0
  fi

  for file in "$defs_dir"/*.json; do
    [ -f "$file" ] || continue
    index_name="$(basename "$file" .json)"
    status="$(curl -s -o /dev/null -w '%{http_code}' "$ELASTICSEARCH_URL/$index_name" || true)"
    if [ "$status" = "200" ]; then
      log "Elasticsearch index $index_name already exists"
      continue
    fi
    log "Creating Elasticsearch index $index_name"
    curl -fsS -X PUT "$ELASTICSEARCH_URL/$index_name" \
      -H 'Content-Type: application/json' \
      --data-binary "@$file"
  done

  log "Elasticsearch bootstrap complete"
}

verify_s3_endpoint_optional() {
  if [ "${ARTCRAFT_SKIP_S3_BOOTSTRAP:-false}" = "true" ]; then
    log "Skipping S3 endpoint check (ARTCRAFT_SKIP_S3_BOOTSTRAP=true)"
    return 0
  fi

  if [ -z "${S3_COMPATIBLE_ENDPOINT_URL:-}" ]; then
    log "S3_COMPATIBLE_ENDPOINT_URL unset; skipping storage check"
    return 0
  fi

  log "Checking S3-compatible endpoint ${S3_COMPATIBLE_ENDPOINT_URL}"
  if ! curl -fsS -o /dev/null -m 15 "${S3_COMPATIBLE_ENDPOINT_URL%/}/"; then
    log "WARN: S3 endpoint not reachable; create required buckets before uploads"
  fi
}

main() {
  log "Starting bootstrap (UNCLOUD_HOOK_PRE_DEPLOY=${UNCLOUD_HOOK_PRE_DEPLOY:-})"
  run_mysql_migrations
  ensure_elasticsearch_indexes
  verify_s3_endpoint_optional
  log "Bootstrap finished successfully"
}

main "$@"
