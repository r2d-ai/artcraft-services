#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${BASE_URL:-http://localhost:8080}"

curl -fsS "${BASE_URL}/" >/dev/null
curl -fsS "${BASE_URL}/v1/auth/config" | grep -q '"success":true'
curl -fsS -o /dev/null -w "%{http_code}" "${BASE_URL}/v1/service/health" | grep -qE '200|503'

echo "smoke checks passed for ${BASE_URL}"
