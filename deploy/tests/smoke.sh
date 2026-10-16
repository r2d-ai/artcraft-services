#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${ARTCRAFT_SMOKE_BASE_URL:-https://studio.example.com}"

echo "Smoke: GET ${BASE_URL}/_status"
curl -fsS "${BASE_URL}/_status" | head -c 200
echo

echo "Smoke: GET ${BASE_URL}/ (web)"
curl -fsS -o /dev/null -w "HTTP %{http_code}\n" "${BASE_URL}/"

echo "Smoke OK"
