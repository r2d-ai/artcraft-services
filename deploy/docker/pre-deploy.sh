#!/bin/bash
set -euo pipefail

if [ -z "${MYSQL_URL:-}" ]; then
  echo "MYSQL_URL is required for migrations" >&2
  exit 1
fi

MIGRATIONS_DIR="${MIGRATIONS_DIR:-/migrations}"
if [ ! -d "$MIGRATIONS_DIR" ]; then
  echo "Migrations directory not found: $MIGRATIONS_DIR" >&2
  exit 1
fi

mysql_run() {
  mysql --protocol=TCP -h"${MYSQL_HOST}" -P"${MYSQL_PORT}" -u"${MYSQL_USER}" -p"${MYSQL_PASSWORD}" "${MYSQL_DATABASE}" "$@"
}

parse_mysql_url() {
  local url="$1"
  url="${url#mysql://}"
  MYSQL_USER="${url%%:*}"
  local rest="${url#*:}"
  MYSQL_PASSWORD="${rest%%@*}"
  rest="${rest#*@}"
  MYSQL_HOST="${rest%%/*}"
  MYSQL_PORT="3306"
  if [[ "$MYSQL_HOST" == *:* ]]; then
    MYSQL_PORT="${MYSQL_HOST#*:}"
    MYSQL_HOST="${MYSQL_HOST%%:*}"
  fi
  MYSQL_DATABASE="${rest#*/}"
  MYSQL_DATABASE="${MYSQL_DATABASE%%\?*}"
}

parse_mysql_url "$MYSQL_URL"

mysql_run -e "CREATE TABLE IF NOT EXISTS __diesel_schema_migrations (
  version VARCHAR(50) PRIMARY KEY NOT NULL,
  run_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP)"

for dir in "$MIGRATIONS_DIR"/*/; do
  name="$(basename "$dir")"
  version="${name%%_*}"
  version="${version//-/}"
  if mysql_run -N -e "SELECT 1 FROM __diesel_schema_migrations WHERE version = '${version}'" | grep -q 1; then
    continue
  fi
  echo "Applying migration ${name}"
  mysql_run < "${dir}up.sql"
  mysql_run -e "INSERT INTO __diesel_schema_migrations (version) VALUES ('${version}')"
done

echo "Migrations complete."
