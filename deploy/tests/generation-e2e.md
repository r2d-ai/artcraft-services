# Generation E2E checklist

1. Configure `INTERNAL_PROVIDER_URL` and `INTERNAL_PROVIDER_ROUTED_IMAGE_MODELS`.
2. Deploy `artcraft-worker-internal` (or enable synchronous routing when available).
3. Call `POST /v1/omni_gen/generate/image` with a routed model and verify output lands in media library.
4. Confirm `SELF_HOSTED_SKIP_CREDIT_BILLING=true` does not debit wallet for internal jobs.
