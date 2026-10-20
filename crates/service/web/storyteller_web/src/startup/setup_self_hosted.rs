use std::collections::BTreeMap;
use std::iter::Iterator;

use errors::AnyhowResult;
use internal_provider_client::InternalProviderClient;

use crate::startup::setup_inference_providers::provider_env_string;

#[derive(Clone, Debug)]
pub struct SelfHostedConfig {
  pub enabled: bool,
  pub skip_credit_billing: bool,
  pub disable_public_signup: bool,
  pub internal_provider: Option<InternalProviderConfig>,
}

#[derive(Clone, Debug)]
pub struct InternalProviderConfig {
  pub client: InternalProviderClient,
  pub routed_image_models: Vec<String>,
  /// Optional per-model rename: ArtCraft internal id → model name sent to the
  /// provider, e.g. `INTERNAL_PROVIDER_MODEL_MAP=minimax_image=image-01`.
  /// Unmapped ids are passed through unchanged.
  pub model_aliases: BTreeMap<String, String>,
}

pub fn setup_self_hosted_config() -> AnyhowResult<SelfHostedConfig> {
  let enabled = easyenv::get_env_bool_or_default("SELF_HOSTED", false);
  let skip_credit_billing = easyenv::get_env_bool_or_default("SELF_HOSTED_SKIP_CREDIT_BILLING", enabled);
  let disable_public_signup = easyenv::get_env_bool_or_default("OIDC_ONLY", false);

  let internal_provider = setup_internal_provider()?;

  Ok(SelfHostedConfig {
    enabled,
    skip_credit_billing,
    disable_public_signup,
    internal_provider,
  })
}

fn setup_internal_provider() -> AnyhowResult<Option<InternalProviderConfig>> {
  let base_url = easyenv::get_env_string_optional("INTERNAL_PROVIDER_URL")
    .filter(|value| !value.is_empty() && value != "unset");

  if base_url.is_none() {
    return Ok(None);
  }

  let base_url = base_url.unwrap();
  internal_provider_client::InternalProviderClient::validate_base_url(&base_url)?;

  let api_key = provider_env_string("INTERNAL_PROVIDER_API_KEY")?;
  let default_model = easyenv::get_env_string_or_default("INTERNAL_PROVIDER_IMAGE_MODEL", "default");
  let routed_models_raw = easyenv::get_env_string_or_default(
    "INTERNAL_PROVIDER_ROUTED_IMAGE_MODELS",
    "flux_1_dev",
  );
  let routed_image_models = routed_models_raw
    .split(',')
    .map(|entry| entry.trim().to_string())
    .filter(|entry| !entry.is_empty())
    .collect();

  let model_aliases = parse_model_aliases(
    easyenv::get_env_string_optional("INTERNAL_PROVIDER_MODEL_MAP"),
  );

  Ok(Some(InternalProviderConfig {
    client: InternalProviderClient::new(base_url, api_key, default_model),
    routed_image_models,
    model_aliases,
  }))
}

/// Parse `INTERNAL_PROVIDER_MODEL_MAP` — comma-separated `artcraft_id=provider_model` pairs.
/// Malformed pairs are skipped with a warning; an empty/missing value yields no aliases.
fn parse_model_aliases(raw: Option<String>) -> BTreeMap<String, String> {
  let mut aliases = BTreeMap::new();
  if let Some(raw) = raw {
    for pair in raw.split(',') {
      let pair = pair.trim();
      if pair.is_empty() {
        continue;
      }
      let Some((internal_id, provider_model)) = pair.split_once('=') else {
        log::warn!(
          "INTERNAL_PROVIDER_MODEL_MAP entry '{}' is malformed — expected 'artcraft_id=provider_model'; skipping",
          pair
        );
        continue;
      };
      let internal_id = internal_id.trim();
      let provider_model = provider_model.trim();
      if internal_id.is_empty() || provider_model.is_empty() {
        log::warn!(
          "INTERNAL_PROVIDER_MODEL_MAP entry '{}' has an empty side — skipping",
          pair
        );
        continue;
      }
      aliases.insert(internal_id.to_string(), provider_model.to_string());
    }
  }
  aliases
}
