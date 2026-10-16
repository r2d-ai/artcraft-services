# Auth isolation checklist

1. Provision two Keycloak users in different groups.
2. User A creates a private media file and inference job.
3. User B must receive `401/403` on direct API access to A's job token, media token, and folder paths.
4. OIDC accounts must link only by `(issuer, subject)` — changing email in IdP must not attach to an unrelated local account.
