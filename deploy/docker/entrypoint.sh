#!/bin/sh
set -eu

if [ -f /usr/share/nginx/html/runtime-config.js ]; then
  :
else
  cat >/usr/share/nginx/html/runtime-config.js <<'EOF'
window.__ARTCRAFT_RUNTIME_CONFIG__ = {
  apiOrigin: window.location.origin,
};
EOF
fi

exec "$@"
