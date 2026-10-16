#!/bin/sh
set -eu

CONFIG_PATH="/usr/share/nginx/html/config.js"
API_PUBLIC_ORIGIN="${ARTCRAFT_API_PUBLIC_ORIGIN:-}"

cat > "$CONFIG_PATH" <<EOF
window.__ARTCRAFT_RUNTIME_CONFIG__ = {
  apiOrigin: "${API_PUBLIC_ORIGIN}"
};
EOF
