# Self-Host Customization Plan (internal deployment)

Tài liệu này là kế hoạch tổng thể để biến self-host ArtCraft thành bản **nội bộ** (OIDC-only, không public signup, không pricing/upgrade, lưu trữ MinIO/S3 ngoài, danh sách model/provider qua file config, quản lý quyền theo user/group). Ưu tiên **custom provider** để test tính năng trước.

Trạng thái hiện tại của môi trường k7 (đã đạt được trong các bước trước):

- API self-host chạy tại `k7:12345` (container `artcraft-selfhost-storyteller-web-1`), healthy, OIDC enabled.
- Webapp dev tại `k7:4201` (Vite proxy `/v1` → `k7:12345`), CORS workaround qua Origin rewrite.
- SSO (OIDC) login thành công end-to-end (nonce bug đã fix).
- `crypto.randomUUID` shim đã thêm cho dev server non-secure-context.
- Build image qua `docker build --allow=network.host --network=host -f build/service_cpu.Dockerfile`.

---

## 1. Mục tiêu & phạm vi

| # | Mục tiêu | Trạng thái hiện tại |
|---|----------|---------------------|
| 1 | Tắt signup công khai + tắt Google login, chỉ OIDC | `OIDC_ONLY=true` đã tắt signup. Google login button vẫn render trong `signup-form.tsx` và `signup-cta-modal.tsx`. |
| 2 | Danh sách model/provider cho image/video/audio configurable qua file | Hardcode trong `configs/omni_gen/{image,video,audio,mesh,splat}_models.rs`. |
| 3 | Storage MinIO/S3 ngoài | Đã S3-compatible qua env (`S3_COMPATIBLE_ENDPOINT_URL`, `ACCESS_KEY`, `SECRET_KEY`, `REGION_NAME`, buckets). MinIO nên hoạt động nếu trỏ env tới MinIO. |
| 4 | Quản lý permission ai/group nào dùng tính năng nào | `user_roles` table có cờ feature (TTS/W2L). OmniGen (image/video/audio) chưa có cờ. |
| 5 | Tắt toàn bộ UI upgrade/plans/pricing | Route `/pricing`, TopBar pricing/upgrade, SignupCtaModal navigate `/pricing`. |
| 6 | Ưu tiên custom provider để test | `INTERNAL_PROVIDER_URL` + `INTERNAL_PROVIDER_ROUTED_IMAGE_MODELS` đã tồn tại, nhưng synchronous routing trả 500 ("deploy artcraft-worker-internal"). |

---

## 2. Phase 0 — Custom provider để test tính năng (ưu tiên cao nhất)

**Mục tiêu**: Có 1 provider tuỳ chỉnh (OpenAI-compatible `/v1/images/generations`) để test generate end-to-end mà không phụ thuộc FAL key.

### Hiện trạng code

- `setup_self_hosted.rs`: đọc `INTERNAL_PROVIDER_URL`, `INTERNAL_PROVIDER_API_KEY`, `INTERNAL_PROVIDER_IMAGE_MODEL`, `INTERNAL_PROVIDER_ROUTED_IMAGE_MODELS`.
- `run_pipeline_v2.rs:should_route_to_internal_provider()` → route model theo danh sách vào `RouterProvider::Fal` (placeholder), rồi `finalize_and_generate` trả 500 "synchronous routing is not enabled in this build; deploy artcraft-worker-internal".
- `internal_provider_client::InternalProviderClient::generate_image_png_bytes` gọi OpenAI-compatible endpoint và trả PNG bytes, nhưng **không được gọi ở đâu** trong pipeline hiện tại.

### Việc cần làm

1. **Wiring synchronous path**: trong `run_pipeline_v2.rs`, khi `should_route_to_internal_provider(model)` true, gọi `server_state.self_hosted_config.internal_provider.client.generate_image_png_bytes(prompt, model_name)` và trả `GenerateImageResponse` với ảnh (base64 PNG → upload lên storage bucket, trả media token). Bỏ nhánh 500.
2. **Cost estimate**: `estimate_cost_in_credits` đang ép `RouterProvider::Artcraft` → provider custom phải bypass cost (self-host skip billing) → trả 0 credits / is_free=true.
3. **Model list**: thêm 1 model entry trỏ tới custom provider (ví dụ `custom_internal_image`) trong `image_models.rs`, hoặc reuse `flux_1_dev` và route qua `INTERNAL_PROVIDER_ROUTED_IMAGE_MODELS=flux_1_dev`.
4. **Env**: set `INTERNAL_PROVIDER_URL`, `INTERNAL_PROVIDER_API_KEY`, `INTERNAL_PROVIDER_IMAGE_MODEL`, `INTERNAL_PROVIDER_ROUTED_IMAGE_MODELS` trong `.env`; test với server OpenAI-compatible (LM Studio / vLLM / ComfyUI wrapper / mock).
5. **Video/audio**: tương tự — hiện chưa có internal provider path cho video/audio. Nếu chỉ test image trước, đủ. Nếu cần video/audio custom, thêm `InternalProviderClient` methods + pipeline wiring cho video/audio (chưa có skeleton).

### Definition of done Phase 0

- `INTERNAL_PROVIDER_URL=http://<custom-host>:port` + routed model → generate image thành công, ảnh hiển thị trong UI, không cần FAL key.
- Cost estimate trả 0 credits (self-host skip billing).

---

## 3. Phase 1 — OIDC-only, tắt signup & Google login

### Backend (đã gần xong)

- `OIDC_ONLY=true` → `disable_public_signup=true` → `auth_config_handler` trả `oidc_only: true`.
- `/v1/auth/config` đã trả `oidc_enabled: true`.

### Frontend

- `signup-form.tsx` và `signup-cta-modal.tsx` render `<GoogleLoginButton />` → xoá/conditional theo `authConfig.oidc_only` (fetch `/v1/auth/config`).
- `/signup` route (app.tsx line 154) → redirect về `/login` khi `oidc_only` (hoặc xoá route).
- `SignupCtaModal` (app.tsx line 223) → không mount khi self-host/oidc_only.
- Login page: nếu `oidc_only`, ẩn form email/password, chỉ hiện nút SSO (đã có `OidcLoginButton`).
- `/set-password`, `/forgot-password*` → có thể giữ cho OIDC providers hỗ trợ password reset, nhưng với OIDC-only nên redirect.

### Definition of done Phase 1

- Truy cập `/login` chỉ thấy "Sign in with SSO"; `/signup` redirect; không còn Google button; signup CTA modal không hiện.

---

## 4. Phase 2 — Tắt UI upgrade/plans/pricing

### Frontend

- `app.tsx`: xoá route `/pricing` (hoặc redirect `/`).
- `TopBarActions.tsx`: xoá Pricing link (lines 108, 253), Upgrade CTA (119-130, 334-338), `hasActiveSubscription` billing check (54-56).
- `signup-cta-modal.tsx`: xoá navigate `/pricing` (line 63); modal chỉ hiện CTA "Sign in".
- `breadcrumbs.tsx`: xoá `/pricing` entry.
- `CreateMediaPageShell.tsx`: empty-state CTA không hiện signup.
- Feature flag: đọc `authConfig.self_hosted` (đã trả từ backend) để conditionally ẩn, tránh hardcode — cho phép cùng codebase chạy cả hosted và self-host.

### Backend

- Không cần thay đổi; billing endpoints vẫn tồn tại nhưng frontend không gọi. Có thể gate billing routes sau `SELF_HOSTED` env nếu muốn.

### Definition of done Phase 2

- Top bar không có Pricing/Upgrade; `/pricing` redirect; không có modal đẩy sang billing.

---

## 5. Phase 3 — Danh sách model/provider qua file config

### Hiện trạng

- `configs/omni_gen/{image,video,audio,mesh,splat}_models.rs` hardcode Rust structs.
- Enum `CommonImageModel`/`CommonVideoModel`/... trong `crates/schema/public/enums`.
- `hydrate_to_router_request.rs` map API enum → `RouterImageModel`.

### Kế hoạch

1. **Bước nhỏ (không refactor lớn)**: cho phép override qua env var `OMNI_GEN_IMAGE_MODELS_FILE` (JSON) đọc lúc startup, fallback về hardcoded list nếu không set. Tương tự video/audio/mesh/splat.
2. Schema JSON: array of `{ model, full_name, text_prompt_supported, aspect_ratio_options, aspect_ratio_default, resolution_options, resolution_default, quality_options, quality_default, batch_size_min/max/default, providers: [{ provider, quality_options? }] }`.
3. Validate model string → enum variant; skip unknown với warning log (không crash).
4. Cost handler: `estimate_cost_in_credits` ép `RouterProvider::Artcraft` → với self-host + custom provider, trả 0 credits (skip billing) — đã ok.
5. Long-term: chuyển hardcoded list sang file config mặc định trong `includes/` (đã COPY vào image) để edit không cần recompile? Rust compile-time Lazy cần rebuild. Nếu muốn runtime-editable, load JSON at startup (Phase 3.1).

### Definition of done Phase 3

- Set `OMNI_GEN_IMAGE_MODELS_FILE=/configs/image_models.json` → backend load list từ file; thêm/bớt model không cần recompile Rust (chỉ restart container).
- UI hiển thị đúng danh sách mới.

---

## 6. Phase 4 — Storage MinIO / S3 ngoài

### Hiện trạng

- Đã S3-compatible: `S3_COMPATIBLE_ENDPOINT_URL`, `ACCESS_KEY`, `SECRET_KEY`, `REGION_NAME`, buckets `W2L_PRIVATE_DOWNLOAD_BUCKET_NAME`, `W2L_PUBLIC_DOWNLOAD_BUCKET_NAME`, `GC_ENABLED_PUBLIC_BUCKET_NAME`.
- SeaweedFS trong compose là S3-compatible; MinIO cũng vậy.

### Việc cần làm

1. Đổi env trong `build/selfhost/.env`:
   ```
   S3_COMPATIBLE_ENDPOINT_URL=http://minio:9000
   ACCESS_KEY=<minio-access-key>
   SECRET_KEY=<minio-secret-key>
   REGION_NAME=us-east-1
   W2L_PRIVATE_DOWNLOAD_BUCKET_NAME=artcraft-private
   W2L_PUBLIC_DOWNLOAD_BUCKET_NAME=artcraft-public
   GC_ENABLED_PUBLIC_BUCKET_NAME=artcraft-public-gc
   ```
2. Tạo buckets trên MinIO (mc mb ...).
3. CORS trên MinIO bucket (cho upload từ browser): `mc anonymous set download` hoặc policy JSON.
4. Nếu MinIO dùng HTTPS với self-signed cert — backend reqwest có thể reject cert; cần CA trust hoặc `S3...` có option skip verify (kiểm tra crate `s3` client config).
5. Webhook FAL: `FAL_WEBHOOK_URL` phải trỏ về URL public mà FAL truy cập được (không phải localhost). Nếu không có public URL, job completion có thể polling thay vì webhook — kiểm tra `fal_request` flow.

### Definition of done Phase 4

- Upload media + generate image lưu file thành công vào MinIO; download URL trả về đúng; không còn phụ thuộc SeaweedFS.

---

## 7. Phase 5 — Quản lý permission ai/group dùng tính năng nào

### Hiện trạng

- `user_roles` table có cờ feature cho TTS/W2L (can_use_tts, can_use_w2l, ...).
- OmniGen (image/video/audio) **chưa** có cờ nào.
- Role gán qua `set_user_role` / `insert_user_role` (admin SQL/manual).

### Kế hoạch

1. **Migration**: thêm cột vào `user_roles`: `can_use_image_gen`, `can_use_video_gen`, `can_use_audio_gen`, `can_use_mesh_gen`, `can_use_splat_gen` (BOOLEAN default TRUE cho self-host).
2. **mysql_queries**: thêm query `get_user_role_permissions_by_token` trả struct permission.
3. **Backend middleware/helper**: `require_feature_access(feature)` kiểm tra role của session user; trả 403 nếu không có quyền.
4. **Apply trong handlers**: omni_gen generate/cost handlers gọi `require_feature_access("image")` (hoặc tương đương) — hoặc filter model list theo permission (model có `required_permission`).
5. **Admin UI/API**: endpoint admin để gán role cho user (đã có `set_user_role`); frontend admin page (hoặc CLI) để toggle permission per role.
6. **OIDC group mapping (tùy chọn)**: khi user login OIDC, map `groups` claim → role slug (ví dụ group `artcraft-image-users` → role `image_user`). Cần thêm logic trong `oidc_callback_handler` sau khi create/link account.

### Definition of done Phase 5

- User thuộc role không có `can_use_video_gen` → gọi `/v1/omni_gen/generate/video` trả 403 với message rõ.
- Admin có thể gán role/permission qua SQL hoặc endpoint.

---

## 8. Phase 6 — Polish & hardening

- CORS: thêm origin `http://k7:4201` (hoặc env `CORS_EXTRA_ALLOWED_ORIGINS`) vào `add_development_only` / `add_artcraft_webapp` để không cần rewrite Origin trong Vite proxy (hiện đang workaround).
- Cookie: `COOKIE_DOMAIN=localhost` ok cho dev; production self-host nên set `COOKIE_DOMAIN` theo domain thật (ví dụ `.internal.company.com`) và `COOKIE_SECURE=true` nếu HTTPS.
- Webhook FAL: nếu không có public URL, cân nhắc polling thay vì webhook (kiểm tra `fal_request`).
- Remove debug logging đã thêm trong `oidc_callback_handler.rs` (hoặc hạ xuống `debug!`) trước khi production.
- Cleanup: `docker-compose.override.yml` (network host) chỉ để build; có thể xoá sau khi build xong.

---

## 9. Thứ tự ưu tiên đề xuất

1. **Phase 0** (custom provider) — đã có skeleton, chỉ cần wiring synchronous path + cost bypass. Test image gen không cần FAL key.
2. **Phase 1 + 2** (OIDC-only + tắt pricing UI) — frontend changes, không cần rebuild backend.
3. **Phase 4** (MinIO) — env change + restart, không rebuild.
4. **Phase 3** (configurable model list) — refactor nhỏ, rebuild backend.
5. **Phase 5** (permissions) — migration + handler gate.
6. **Phase 6** (polish).

---

## 10. Các file chính cần chạm

| Phase | File |
|-------|------|
| 0 | `crates/service/web/storyteller_web/src/http_server/endpoints/omni_gen/generate/image/pipeline_v2/run_pipeline_v2.rs` (wiring internal provider) |
| 0 | `crates/service/web/storyteller_web/src/configs/omni_gen/image_models.rs` (thêm model custom) |
| 1 | `frontend/apps/artcraft-webapp/src/components/auth/signup-form.tsx`, `signup-cta-modal.tsx`, `app.tsx`, `pages/login/login.tsx` |
| 2 | `frontend/apps/artcraft-webapp/src/components/topbar/TopBarActions.tsx`, `app.tsx`, `breadcrumbs.tsx` |
| 3 | `crates/service/web/storyteller_web/src/configs/omni_gen/*.rs`, `hydrate_to_router_request.rs`, enum `common_image_model.rs` |
| 4 | `build/selfhost/.env` (S3 vars) |
| 5 | migration `user_roles`, `mysql_queries` user_roles, handlers omni_gen generate/cost |
| 6 | `crates/lib/actix_cors_configs/src/configs/development_only.rs` / `artcraft_webapp.rs`, `oidc_callback_handler.rs` (log) |

---

## 11. Lưu ý về build & deploy

- Mọi thay đổi Rust → `docker build --allow=network.host --network=host -f build/service_cpu.Dockerfile -t artcraft-services:local .` rồi `docker compose up -d --no-deps storyteller-web`.
- Frontend changes → restart Vite dev server (hoặc build production).
- DB migration → chạy migrate container hoặc apply SQL thủ công.
- `.env` changes → `docker compose up -d --no-deps storyteller-web` (không cần rebuild).

---

Nếu bạn đồng ý plan này, tôi có thể bắt đầu từ **Phase 0** (wiring custom provider) ngay — đó là phần bạn yêu cầu ưu tiên để test tính năng. Bạn muốn bắt đầu từ phase nào?
