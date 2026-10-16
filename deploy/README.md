# ArtCraft self-hosted / Uncloud deployment

This directory contains Docker images and Uncloud compose manifests for running ArtCraft Studio with:

- Same-origin frontend + API (`artcraft-web` nginx proxies `/v1/*` to `artcraft-api`)
- Optional Keycloak OIDC login (`OIDC_*` environment variables)
- Optional internal OpenAI-compatible inference provider (`INTERNAL_PROVIDER_*`)
- Self-hosted mode (`SELF_HOSTED=true`) with optional credit-billing bypass

## Build

```bash
docker build -f deploy/docker/Dockerfile.backend -t artcraft-api:local .
docker build -f deploy/docker/Dockerfile.web -t artcraft-web:local .
```

## Uncloud (target pipeline)

```bash
cp deploy/uncloud/.env.example deploy/uncloud/.env
# edit deploy/uncloud/.env with real endpoints and secrets

uc build -f deploy/uncloud/compose.app.yaml --push
uc deploy -f deploy/uncloud/compose.app.yaml --no-build
uc inspect artcraft-web
uc inspect artcraft-api
```

`artcraft-api` runs `deploy/docker/pre-deploy.sh` via `x-pre-deploy` before rollout (migrations bundled in the image).

## Smoke test

```bash
BASE_URL=https://studio.example.com deploy/tests/smoke.sh
```
