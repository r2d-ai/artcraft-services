# Image generation vertical slice (phase 4)

End-to-end check for one internal image model after provider adapter configuration.

## Preconditions

- `INTERNAL_PROVIDER_URL` (or equivalent) set on `artcraft-api`.
- Admin granted model access to the test user.
- Worker processes required by the workflow are deployed.

## Flow

1. Authenticate as test user (OIDC session).
2. `POST /v1/omni_gen/generate/image` (or the routed endpoint for the internal model)
   with a fresh `idempotency_token`.
3. Poll job status until `succeeded` or terminal `failed`.
4. Confirm output object exists in company S3 and appears in the user's media library.
5. Download via the app URL path (signed URL or API stream) and verify bytes.

## Failure modes

- Timeout surfaces as failed job, not stuck `running` forever.
- Retry deploy must not double-charge quota or duplicate output for the same idempotency token.
- Cancel must not report success while the engine still runs (if cancel is supported).
