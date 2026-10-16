use beeble_client::creds::beeble_api_key::BeebleApiKey;
use errors::AnyhowResult;
use fal_client::creds::fal_api_key::FalApiKey;

/// All third-party inference provider credentials and configuration grouped together.
pub struct InferenceProviders {
  pub fal: FalData,
  pub gmicloud: GmiCloudData,
  pub grok_api: GrokApiData,
  pub beeble: BeebleData,
  pub kinovi_web: KinoviWebData,
  pub openai: OpenAiData,
  pub worldlabs: WorldLabsData,
}

/// Fal integration
#[derive(Clone)]
pub struct FalData {
  pub api_key: FalApiKey,
  pub webhook_url: String,
}

/// GmiCloud integration
#[derive(Clone)]
pub struct GmiCloudData {
  pub api_key: gmicloud_client::creds::gmicloud_api_key::GmiCloudApiKey,
}

/// Grok (xAI) API integration
#[derive(Clone)]
pub struct GrokApiData {
  pub api_key: grok_api_client::creds::grok_api_key::GrokApiKey,
}

/// Beeble SwitchX integration
#[derive(Clone)]
pub struct BeebleData {
  pub api_key: BeebleApiKey,
  pub webhook_url: String,
}

/// Seedance 2 Pro integration
#[derive(Clone)]
pub struct KinoviWebData {
  pub cookies_volcengine: String,

  pub cookies_byteplus: String,

  pub cookies_byteplus_ultra: String,
}

/// OpenAI integration
#[derive(Clone)]
pub struct OpenAiData {
  pub api_key: String,
}

/// World Labs integration
#[derive(Clone)]
pub struct WorldLabsData {
  pub api_key: String,
}

pub fn provider_env_string(env_name: &str) -> AnyhowResult<String> {
  let self_hosted = easyenv::get_env_bool_or_default("SELF_HOSTED", false);
  if self_hosted {
    Ok(
      easyenv::get_env_string_optional(env_name)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unset".to_string()),
    )
  } else {
    Ok(easyenv::get_env_string_required(env_name)?)
  }
}

pub fn setup_inference_providers() -> AnyhowResult<InferenceProviders> {
  Ok(InferenceProviders {
    fal: FalData {
      api_key: FalApiKey::new(provider_env_string("FAL_API_KEY")?),
      webhook_url: provider_env_string("FAL_WEBHOOK_URL")?,
    },
    gmicloud: GmiCloudData {
      api_key: gmicloud_client::creds::gmicloud_api_key::GmiCloudApiKey::new(
        provider_env_string("GMICLOUD_API_KEY")?,
      ),
    },
    grok_api: GrokApiData {
      api_key: grok_api_client::creds::grok_api_key::GrokApiKey::new(
        provider_env_string("GROK_API_KEY")?,
      ),
    },
    beeble: BeebleData {
      api_key: BeebleApiKey::new(provider_env_string("BEEBLE_API_KEY")?),
      webhook_url: provider_env_string("BEEBLE_WEBHOOK_URL")?,
    },
    kinovi_web: KinoviWebData {
      cookies_volcengine: provider_env_string("SEEDANCE2PRO_COOKIES")?,
      cookies_byteplus: provider_env_string("SEEDANCE2PRO_WHITELIST_COOKIES")?,
      cookies_byteplus_ultra: provider_env_string("SEEDANCE2PRO_BYTEPLUS_ULTRA_COOKIES")?,
    },
    openai: OpenAiData {
      api_key: provider_env_string("OPENAI_API_KEY")?,
    },
    worldlabs: WorldLabsData {
      api_key: provider_env_string("WORLDLABS_API_KEY")?,
    },
  })
}
