use log::{error, info, warn};

use internal_provider_client::InternalProviderClient;
use artcraft_router::api::image_list_ref::ImageListRef;
use artcraft_router::api::router_image_model::RouterImageModel;
use artcraft_router::api::router_provider::RouterProvider;
use artcraft_router::client::router_client::RouterClient;
use artcraft_router::client::router_fal_client::RouterFalClient;
use artcraft_router::client::router_kinovi_web_client::RouterKinoviWebClient;
use artcraft_router::generate::generate_image::generate_image_request_builder::GenerateImageRequestBuilder;
use artcraft_router::generate::generate_image::generate_image_response::GenerateImageResponse;
use artcraft_router::generate::generate_image::image_generation_draft_context::ImageGenerationDraftContext;
use artcraft_router::generate::generate_image::image_generation_draft_or_request::ImageGenerationDraftOrRequest;
use artcraft_router::generate::generate_image::image_generation_request::ImageGenerationRequest;
use kinovi_web_client::creds::kinovi_web_session::KinoviWebSession;
use tokens::tokens::generic_inference_jobs::InferenceJobToken;
use tokens::tokens::users::UserToken;

use crate::billing::wallets::attempt_wallet_deduction::attempt_wallet_deduction_else_common_web_error;
use crate::http_server::common_responses::common_web_error::CommonWebError;
use crate::http_server::endpoints::omni_gen::generate::image::insert_db_job::insert_fal_job::{
  insert_fal_job, InsertFalJobArgs,
};
use crate::http_server::endpoints::omni_gen::generate::image::insert_db_job::shared_job_args::SharedJobArgs;
use bucket_paths::legacy::typified_paths::public::media_files::bucket_file_path::MediaFileBucketPath;
use enums::by_table::media_files::media_file_class::MediaFileClass;
use enums::by_table::media_files::media_file_origin_category::MediaFileOriginCategory;
use enums::by_table::media_files::media_file_type::MediaFileType;
use enums::common::generation_provider::GenerationProvider;
use enums::common::visibility::Visibility;
use hashing::sha256::sha256_hash_bytes::sha256_hash_bytes;
use images::image_info::image_info::ImageInfo;
use mysql_queries::queries::generic_inference::api_providers::fal::mark_fal_generic_inference_job_successfully_done::{
  mark_fal_generic_inference_job_successfully_done, MarkJobArgs,
};
use mysql_queries::queries::media_files::create::insert_builder::media_file_insert_builder::MediaFileInsertBuilder;
use crate::http_server::endpoints::generate::common::generation_debug_logs::{
  insert_provider_request_debug_log, provider_request_debug_log_type, GenerationDebugLogContext,
};
use crate::http_server::endpoints::omni_gen::generate::image::pipeline_result::ImagePipelineResult;
use crate::http_server::endpoints::omni_gen::shared_utils::kinovi_account::KinoviAccount;
use crate::http_server::endpoints::omni_gen::shared_utils::map_kinovi_web_router_error::map_router_error_to_web_error;
use crate::state::server_state::ServerState;
use sqlx::Acquire;
use crate::util::lookup::lookup_media_files_as_cdn_url_list_and_map::MediaFilesAsCdnUrlListAndMap;

pub struct RunPipelineV2Args<'a> {
  pub router_builder: &'a GenerateImageRequestBuilder,
  /// API model id from the request (`model` JSON field), including custom self-host ids.
  pub api_image_model_id: &'a str,
  pub server_state: &'a ServerState,
  pub user_token: &'a UserToken,
  pub ip_address: &'a str,
  pub resolved_media: &'a MediaFilesAsCdnUrlListAndMap,
  pub kinovi_account: KinoviAccount,
  pub debug_log_context: &'a GenerationDebugLogContext<'a>,
  /// The handler's open connection. The pipeline uses it for its remaining
  /// pre-request DB writes (billing, outbound-request debug log) and releases
  /// it BEFORE the external provider call.
  pub mysql_connection: sqlx::pool::PoolConnection<sqlx::MySql>,
}

// NB: This pipeline does an external generation call (`finalize_and_generate`) that can take many
// seconds. It deliberately does NOT hold a pooled DB connection across that call — it acquires a
// short-lived connection only for the wallet deduction. Holding a pooled connection across the
// external call is what starves the pool and causes `PoolTimedOut`.
pub async fn run_pipeline_v2(
  args: RunPipelineV2Args<'_>,
) -> Result<ImagePipelineResult, CommonWebError> {
  let RunPipelineV2Args {
    router_builder,
    api_image_model_id,
    server_state,
    user_token,
    ip_address,
    resolved_media,
    kinovi_account,
    debug_log_context,
    mut mysql_connection,
  } = args;

  let hydrated_builder = apply_hydrated_media_inputs(
    router_builder,
    resolved_media,
  );

  let mut cost = estimate_cost_in_credits(api_image_model_id, &hydrated_builder, server_state)?;
  if server_state.self_hosted_config.skip_credit_billing {
    cost = 0;
  }

  info!("Charging wallet: {} credits", cost);

  let apriori_job_token = InferenceJobToken::generate();

  // ==================== SELF-HOST / CATALOG PROVIDER — SYNCHRONOUS PATH ==================== //
  if uses_self_hosted_image_backend(api_image_model_id, server_state) {
    let (client, provider_model) =
      resolve_openai_compatible_image_client(api_image_model_id, server_state)?;

    let prompt = router_builder.prompt.clone().unwrap_or_default();

    info!(
      "Catalog/internal provider: generating image for model {} (sent as '{}')",
      api_image_model_id, provider_model
    );

    let png_bytes = client
      .generate_image_png_bytes(&prompt, Some(&provider_model))
      .await
      .map_err(|err| {
        warn!("Internal provider image generation failed: {:#}", err);
        CommonWebError::from_anyhow_error(err)
      })?;

    let file_hash = sha256_hash_bytes(&png_bytes)
      .map_err(CommonWebError::from_anyhow_error)?;
    let image_info = ImageInfo::decode_image_from_bytes(&png_bytes)
      .map_err(CommonWebError::from_error)?;
    let mime_type = "image/png";
    let extension_with_period = ".png";
    let public_upload_path =
      MediaFileBucketPath::generate_new(Some("artcraft_"), Some(extension_with_period));

    server_state
      .public_bucket_client
      .upload_file_with_content_type_process(
        public_upload_path.get_full_object_path_str(),
        &png_bytes,
        mime_type,
      )
      .await
      .map_err(|err| {
        warn!("Internal provider upload to bucket failed: {:#}", err);
        CommonWebError::from_anyhow_error(err)
      })?;

    let media_token = MediaFileInsertBuilder::new()
      .checksum_sha2(&file_hash)
      .creator_ip_address(ip_address)
      .file_size_bytes(png_bytes.len() as u64)
      .frame_height(image_info.height())
      .frame_width(image_info.width())
      .maybe_creator_user(Some(user_token))
      .maybe_generation_provider(Some(GenerationProvider::Artcraft))
      .maybe_prompt_token(None)
      .maybe_source_job_token(Some(&apriori_job_token))
      .media_file_class(MediaFileClass::Image)
      .media_file_origin_category(MediaFileOriginCategory::ThirdPartyInference)
      .media_file_type(MediaFileType::try_from_mime_type(mime_type).unwrap_or(MediaFileType::Png))
      .mime_type(mime_type)
      .public_bucket_directory_hash(&public_upload_path)
      .insert_pool(&server_state.mysql_pool)
      .await
      .map_err(|err| {
        warn!("Internal provider media file insert failed: {:?}", err);
        CommonWebError::from_error(err)
      })?;

    info!("Internal provider uploaded media file: {}", media_token);

    let mut transaction = mysql_connection.begin().await.map_err(|err| {
      error!("Error starting MySQL transaction (internal): {:?}", err);
      CommonWebError::from_error(err)
    })?;

    insert_fal_job(InsertFalJobArgs {
      external_job_id: "internal",
      shared: SharedJobArgs {
        apriori_job_token: &apriori_job_token,
        idempotency_token: router_builder
          .idempotency_token
          .as_deref()
          .unwrap_or(""),
        user_token,
        maybe_avt_token: None,
        maybe_prompt_token: None,
        maybe_debug_log_event_token: None,
        maybe_model_type: None,
        maybe_platform_type: None,
        ip_address,
        transaction: &mut transaction,
      },
    })
    .await
    .map_err(|err| {
      warn!("Internal provider job insert failed: {:?}", err);
      CommonWebError::from_error(err)
    })?;

    mark_fal_generic_inference_job_successfully_done(MarkJobArgs {
      job_token: &apriori_job_token,
      media_file_token: &media_token,
      maybe_batch_token: None,
      mysql_executor: &mut *transaction,
      phantom: Default::default(),
    })
    .await
    .map_err(|err| {
      warn!("Internal provider mark-success failed: {:#}", err);
      CommonWebError::from_anyhow_error(err)
    })?;

    transaction.commit().await.map_err(|err| {
      error!("Error committing internal provider transaction: {:?}", err);
      CommonWebError::from_error(err)
    })?;

    return Ok(ImagePipelineResult {
      apriori_job_token: apriori_job_token.clone(),
      response: GenerateImageResponse::Artcraft(
        artcraft_router::generate::generate_image::generate_image_response::ArtcraftImageResponsePayload {
          inference_job_token: apriori_job_token,
        },
      ),
      maybe_wallet_ledger_entry_token: None,
    });
  }

  let draft_or_request = build_execution_request(&hydrated_builder, server_state)?;

  let maybe_wallet_ledger_entry_token = if cost > 0 {
    // Billed on the handler's connection (same pre-request DB phase).
    let deduction = attempt_wallet_deduction_else_common_web_error(
      user_token,
      Some(apriori_job_token.as_str()),
      cost,
      &mut mysql_connection,
    ).await?;
    Some(deduction.ledger_entry_token)
  } else {
    None
  };

  // Debug-log the outbound provider request BEFORE the send — still on the
  // handler's connection — so the payload is captured even when the
  // upload/enqueue fails.
  if let Some(debug_log_type) = provider_request_debug_log_type(draft_or_request.get_provider()) {
    insert_provider_request_debug_log(
      debug_log_context,
      debug_log_type,
      &format!("{:#?}", draft_or_request),
      &mut *mysql_connection,
    ).await;
  }

  // NB: Done with pre-request DB writes. Release the pooled connection before
  // the (slow, external) provider call — holding it across that call is what
  // starves the pool and causes PoolTimedOut. No pooled DB connection is held
  // across the external generation call below.
  drop(mysql_connection);

  let response = finalize_and_generate(
    draft_or_request,
    router_builder,
    server_state,
    resolved_media,
    kinovi_account,
  ).await?;

  Ok(ImagePipelineResult {
    apriori_job_token,
    response,
    maybe_wallet_ledger_entry_token,
  })
}

fn build_execution_request(
  router_builder: &GenerateImageRequestBuilder,
  server_state: &ServerState,
) -> Result<ImageGenerationDraftOrRequest, CommonWebError> {
  let mut execution_builder = router_builder.clone();
  execution_builder.provider = provider_for_model(execution_builder.model, server_state);

  execution_builder.build2().map_err(|e| {
    warn!("Failed to build2 for image v2 pipeline: {}", e);
    CommonWebError::from_error(e)
  })
}

/// Route each image model to the provider that fulfils it. Midjourney and
/// Seedream 5.0 Pro (both variants) are served via the KinoviWeb/Kinovi
/// backend; everything else flows through Fal. Which Kinovi account fulfils
/// a Kinovi request is the handler's `kinovi_account` choice.
fn provider_for_model(model: RouterImageModel, server_state: &ServerState) -> RouterProvider {
  if should_route_to_internal_provider(model, server_state) {
    return RouterProvider::Fal;
  }

  match model {
    RouterImageModel::Midjourney7
    | RouterImageModel::Midjourney7Niji
    | RouterImageModel::Midjourney8
    | RouterImageModel::Seedream5p0Pro
    | RouterImageModel::Seedream5p0ProUltra => RouterProvider::KinoviWeb,
    _ => RouterProvider::Fal,
  }
}

pub fn custom_catalog_routes_model(
  server_state: &ServerState,
  api_image_model_id: &str,
) -> bool {
  server_state
    .omni_gen_catalog
    .custom_image_registry
    .as_ref()
    .and_then(|registry| registry.route_for(api_image_model_id))
    .is_some()
}

pub fn uses_self_hosted_image_backend(
  api_image_model_id: &str,
  server_state: &ServerState,
) -> bool {
  custom_catalog_routes_model(server_state, api_image_model_id)
    || should_route_to_internal_provider_for_api_id(api_image_model_id, server_state)
}

fn resolve_openai_compatible_image_client(
  api_image_model_id: &str,
  server_state: &ServerState,
) -> Result<(InternalProviderClient, String), CommonWebError> {
  if let Some(registry) = &server_state.omni_gen_catalog.custom_image_registry {
    if let Some((client, provider_model)) = registry.openai_client_for_model(api_image_model_id) {
      return Ok((client.clone(), provider_model));
    }
  }

  let internal = server_state
    .self_hosted_config
    .internal_provider
    .as_ref()
    .ok_or_else(|| {
      CommonWebError::server_error_with_message(
        "image model is not configured with a provider backend",
      )
    })?;

  let provider_model = internal
    .model_aliases
    .get(api_image_model_id)
    .cloned()
    .unwrap_or_else(|| api_image_model_id.to_string());

  Ok((internal.client.clone(), provider_model))
}

pub fn should_route_to_internal_provider_for_api_id(
  api_image_model_id: &str,
  server_state: &ServerState,
) -> bool {
  server_state
    .self_hosted_config
    .internal_provider
    .as_ref()
    .map(|config| {
      config
        .routed_image_models
        .iter()
        .any(|entry| entry == api_image_model_id)
    })
    .unwrap_or(false)
}

pub fn should_route_to_internal_provider(model: RouterImageModel, server_state: &ServerState) -> bool {
  let model_name = serde_json::to_string(&model)
    .unwrap_or_default()
    .trim_matches('"')
    .to_string();
  should_route_to_internal_provider_for_api_id(&model_name, server_state)
}

fn apply_hydrated_media_inputs(
  router_builder: &GenerateImageRequestBuilder,
  resolved_media: &MediaFilesAsCdnUrlListAndMap,
) -> GenerateImageRequestBuilder {
  let mut hydrated_builder = router_builder.clone();

  match hydrated_builder.image_inputs.as_ref() {
    Some(ImageListRef::MediaFileTokens(tokens)) if !tokens.is_empty() => {
      hydrated_builder.image_inputs = Some(ImageListRef::Urls(
        resolved_media.ordered_url_list.clone(),
      ));
    },
    _ => {},
  }

  hydrated_builder
}

fn estimate_cost_in_credits(
  api_image_model_id: &str,
  router_builder: &GenerateImageRequestBuilder,
  server_state: &ServerState,
) -> Result<u64, CommonWebError> {
  if uses_self_hosted_image_backend(api_image_model_id, server_state) {
    return Ok(0);
  }
  let mut cost_builder = router_builder.clone();
  cost_builder.provider = RouterProvider::Artcraft;

  let request = cost_builder.build2().map_err(|e| {
    warn!("Failed to build image cost request: {}", e);
    CommonWebError::from_error(e)
  })?;

  let cost = request.estimate_cost().map_err(|e| {
    warn!("Failed to estimate image cost: {}", e);
    CommonWebError::from_error(e)
  })?;

  Ok(cost.cost_in_credits.unwrap_or(0))
}

async fn finalize_and_generate(
  draft_or_request: ImageGenerationDraftOrRequest,
  router_builder: &GenerateImageRequestBuilder,
  server_state: &ServerState,
  resolved_media: &MediaFilesAsCdnUrlListAndMap,
  kinovi_account: KinoviAccount,
) -> Result<GenerateImageResponse, CommonWebError> {
  if should_route_to_internal_provider(router_builder.model, server_state) {
    return Err(CommonWebError::server_error_with_message(
      "internal image provider is configured but synchronous routing is not enabled in this build; deploy artcraft-worker-internal",
    ));
  }

  let provider = draft_or_request.get_provider();
  let client = build_router_client(provider, server_state, kinovi_account)?;

  // Inline the draft finalization so the draft context can borrow `client`.
  let request: ImageGenerationRequest = match draft_or_request {
    ImageGenerationDraftOrRequest::Request(request) => request,
    ImageGenerationDraftOrRequest::Draft(draft) => {
      let draft_context = ImageGenerationDraftContext {
        client: Some(&client),
        media_file_to_artcraft_url_map: Some(&resolved_media.token_to_url_map),
      };
      draft.finalize(draft_context)
        .await
        .map_err(|err| {
          warn!("Failed to finalize image v2 draft: {:?}", err);
          map_router_error_to_web_error(err)
        })?
    }
  };

  request.send_request(&client)
    .await
    .map_err(|err| {
      warn!("v2 image generation failed: {:?}", err);
      map_router_error_to_web_error(err)
    })
}

fn build_router_client(
  provider: RouterProvider,
  server_state: &ServerState,
  kinovi_account: KinoviAccount,
) -> Result<RouterClient, CommonWebError> {
  match provider {
    RouterProvider::Fal => {
      let fal_client = RouterFalClient::new_with_webhook(
        server_state.inference_providers.fal.api_key.clone(),
        server_state.inference_providers.fal.webhook_url.clone(),
      );
      Ok(RouterClient::Fal(fal_client))
    },
    RouterProvider::KinoviWeb => {
      let kinovi_web = &server_state.inference_providers.kinovi_web;
      let cookies = match kinovi_account {
        KinoviAccount::Volcengine => kinovi_web.cookies_volcengine.clone(),
        KinoviAccount::BytePlus => kinovi_web.cookies_byteplus.clone(),
        KinoviAccount::BytePlusUltra => kinovi_web.cookies_byteplus_ultra.clone(),
      };
      let session = KinoviWebSession::from_cookies_string(cookies);
      Ok(RouterClient::KinoviWeb(RouterKinoviWebClient::new(session)))
    },
    other => {
      Err(CommonWebError::server_error_with_message(
        &format!("Unsupported provider for image v2 generation: {:?}", other),
      ))
    },
  }
}
