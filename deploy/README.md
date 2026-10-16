# ArtCraft Uncloud / Docker deployment

Deployment artifacts for self-hosting ArtCraft on [Uncloud](https://uncloud.run/) with
Docker-only build hosts (no Node.js or Rust on the cluster).

## Layout

| Path                         | Purpose                                                |
|------------------------------|--------------------------------------------------------|
| `docker/Dockerfile.web`      | Nx/Vite build + nginx (static + `/v1` reverse proxy)   |
| `docker/Dockerfile.backend`  | Rust API and worker binaries + bootstrap scripts       |
| `docker/pre-deploy.sh`       | Migrations and Elasticsearch indexes (pre-deploy hook) |
| `uncloud/compose.app.yaml`   | Application services (`artcraft-*`)                    |
| `uncloud/compose.infra.yaml` | Optional MySQL, Redis, ES, SeaweedFS                    |
| `uncloud/.env.example`       | Runtime configuration template                         |
| `tests/smoke.sh`             | Basic health checks after deploy                       |

## Prerequisites

- Docker on the build machine.
- Uncloud CLI (`uc`) with pre-deploy hook support.
- Filled `deploy/uncloud/.env` (from `.env.example`).
- For production: managed MySQL, Redis, Elasticsearch, and S3-compatible storage, **or**
  `compose.infra.yaml` with `x-machines` volume placement.

## Build and deploy

From the repository root:

```bash
cp deploy/uncloud/.env.example deploy/uncloud/.env
# Edit deploy/uncloud/.env — set hosts, passwords, and public hostname.

cd deploy/uncloud
uc build -f compose.app.yaml --push
uc deploy -f compose.app.yaml --no-build
```

With bundled infrastructure:

```bash
uc deploy -f compose.infra.yaml -f compose.app.yaml
```

Inspect services:

```bash
uc inspect artcraft-web
uc inspect artcraft-api
```

## Frontend API routing

The web image sets `VITE_ARTCRAFT_SELF_HOSTED=true` at build time. In production the
browser targets **same origin**; nginx proxies `/v1` and `/_status` to `artcraft-api`.

Override with `VITE_API_ORIGIN` at build time or `ARTCRAFT_API_PUBLIC_ORIGIN` at runtime
(`/config.js`).

## Bootstrap

`artcraft-api` runs `/opt/artcraft/pre-deploy.sh` via `x-pre_deploy` before rollout:

1. MySQL migrations from `/opt/artcraft/migrations` (idempotent).
2. Elasticsearch indexes from `/opt/artcraft/elasticsearch/index_definitions`.
3. Optional S3 endpoint reachability check.

Set `MYSQL_MIGRATION_*` to the DDL-capable user only if migrations should use a
different account than runtime `MYSQL_URL` (recommended: migration user with DDL,
runtime user without).

## Workers

`artcraft-worker-video-thumbnail` reuses the backend image. Add more workers by copying
the service block and changing `command` to the desired binary.

## Roadmap (spec v0.1)

| Phase | Status in this repo                                      |
|-------|----------------------------------------------------------|
| 1. Docker baseline | Dockerfiles, compose, same-origin web, pre-deploy |
| 2. Self-host cleanup | Optional providers, billing bypass — not yet        |
| 3. Keycloak OIDC | Env placeholders only                               |
| 4. Internal AI adapter | Env placeholders only                           |

See `tests/auth-isolation.md` and `tests/generation-e2e.md` for acceptance criteria
for later phases.

## Local validation (Docker Compose)

You can still use `build/selfhost/` for all-in-one local API testing. The Uncloud
manifests target cluster deployment with a separate static frontend and pre-deploy
migrations instead of `depends_on: service_completed_successfully`.
