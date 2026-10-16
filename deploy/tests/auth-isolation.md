# Auth isolation acceptance (phase 3)

Run against a deployed environment with Keycloak OIDC enabled and at least two users
in different groups.

## Preconditions

- `OIDC_*` variables configured on `artcraft-api`.
- Public signup and password login disabled when OIDC-only mode is on.
- Each user has a distinct session cookie after login.

## API checks

For each pair `(user_a, user_b)`:

1. User A creates a job or uploads media; record `job_token` / `media_file_token`.
2. User B calls `GET /v1/...` for A's tokens directly (no UI). Expect **403** or **404**, never **200** with A's payload.
3. Repeat for folder and asset endpoints documented in the API.

## Session revocation

1. Disable user A in Keycloak.
2. Within the configured session TTL, A's existing cookie must stop authorizing writes.
3. A must not read B's assets after re-login as a different subject.

## IDOR regression

Automate with `curl` and saved cookie jars; do not rely on UI-only tests.
