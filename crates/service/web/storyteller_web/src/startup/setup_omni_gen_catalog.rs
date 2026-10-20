use std::collections::HashSet;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context};
use artcraft_api_defs::omni_gen::models::omni_gen_audio_models::OmniGenAudioModelsResponse;
use artcraft_api_defs::omni_gen::models::omni_gen_image_models::OmniGenImageModelsResponse;
use artcraft_api_defs::omni_gen::models::omni_gen_mesh_models::OmniGenMeshModelsResponse;
use artcraft_api_defs::omni_gen::models::omni_gen_splat_models::OmniGenSplatModelsResponse;
use artcraft_api_defs::omni_gen::models::omni_gen_video_models::OmniGenVideoModelsResponse;
use errors::AnyhowResult;
use log::info;
use serde::Deserialize;
use serde_json::Value;

use crate::configs::omni_gen::audio_models::OMNI_GEN_AUDIO_MODELS_AND_PROVIDERS;
use crate::configs::omni_gen::catalog_allowlist::{
  allowlist_from_model_ids, filter_audio_catalog, filter_image_catalog, filter_mesh_catalog,
  filter_splat_catalog, filter_video_catalog, known_audio_model_ids, known_image_model_ids,
  known_mesh_model_ids, known_splat_model_ids, known_video_model_ids, warn_unknown_catalog_ids,
};
use crate::configs::omni_gen::custom_image_catalog::{
  build_custom_image_models_response, CustomImageCatalogEntry,
};
use crate::configs::omni_gen::custom_image_provider_registry::{
  build_custom_image_registry, CustomImageProviderDefinition, CustomImageRegistry,
};
use crate::startup::setup_inference_providers::provider_env_string;
use crate::configs::omni_gen::image_models::OMNI_GEN_IMAGE_MODELS_AND_PROVIDERS;
use crate::configs::omni_gen::mesh_models::OMNI_GEN_MESH_MODELS_AND_PROVIDERS;
use crate::configs::omni_gen::splat_models::OMNI_GEN_SPLAT_MODELS_AND_PROVIDERS;
use crate::configs::omni_gen::video_models::video_models::OMNI_GEN_VIDEO_MODELS_AND_PROVIDERS;
use crate::startup::setup_self_hosted::SelfHostedConfig;

const ENV_OMNI_GEN_CATALOG_FILE: &str = "OMNI_GEN_CATALOG_FILE";

/// Resolved omni-gen model catalogs served by `GET /v1/omni_gen/models/*`.
#[derive(Clone)]
pub struct OmniGenCatalog {
  pub image: OmniGenImageModelsResponse,
  pub video: OmniGenVideoModelsResponse,
  pub audio: OmniGenAudioModelsResponse,
  pub mesh: OmniGenMeshModelsResponse,
  pub splat: OmniGenSplatModelsResponse,
  /// Multi-provider custom image routes from `OMNI_GEN_CATALOG_FILE` (version 3+).
  pub custom_image_registry: Option<CustomImageRegistry>,
}

#[derive(Debug, Default, Deserialize)]
struct OmniGenCatalogAllowlistFile {
  image: Option<Vec<String>>,
  video: Option<Vec<String>>,
  audio: Option<Vec<String>>,
  mesh: Option<Vec<String>>,
  splat: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct OmniGenCatalogCustomFile {
  version: Option<u32>,
  providers: Option<Vec<CustomImageProviderFile>>,
  image: Vec<CustomImageCatalogEntryFile>,
  video: Option<Vec<String>>,
  audio: Option<Vec<String>>,
  mesh: Option<Vec<String>>,
  splat: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct CustomImageProviderFile {
  id: String,
  r#type: String,
  base_url: String,
  api_key: Option<String>,
  api_key_env: Option<String>,
  default_model: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CustomImageCatalogEntryFile {
  id: String,
  full_name: Option<String>,
  provider: Option<String>,
  provider_model: Option<String>,
}

pub fn setup_omni_gen_catalog(mut self_hosted_config: SelfHostedConfig) -> AnyhowResult<(OmniGenCatalog, SelfHostedConfig)> {
  let base_image = (*OMNI_GEN_IMAGE_MODELS_AND_PROVIDERS).clone();
  let base_video = (*OMNI_GEN_VIDEO_MODELS_AND_PROVIDERS).clone();
  let base_audio = (*OMNI_GEN_AUDIO_MODELS_AND_PROVIDERS).clone();
  let base_mesh = (*OMNI_GEN_MESH_MODELS_AND_PROVIDERS).clone();
  let base_splat = (*OMNI_GEN_SPLAT_MODELS_AND_PROVIDERS).clone();

  let maybe_path = easyenv::get_env_string_optional(ENV_OMNI_GEN_CATALOG_FILE)?;
  let maybe_path = match maybe_path {
    None => {
      return Ok((OmniGenCatalog {
        image: base_image,
        video: base_video,
        audio: base_audio,
        mesh: base_mesh,
        splat: base_splat,
        custom_image_registry: None,
      }, self_hosted_config));
    }
    Some(path) => path.trim().to_string(),
  };

  if maybe_path.is_empty() {
    return Err(anyhow!("{} is set but empty", ENV_OMNI_GEN_CATALOG_FILE));
  }

  let path = Path::new(&maybe_path);
  let contents = fs::read_to_string(path)
    .with_context(|| format!("reading {} ({})", ENV_OMNI_GEN_CATALOG_FILE, path.display()))?;
  let root: Value = serde_json::from_str(&contents)
    .with_context(|| format!("parsing JSON in {}", path.display()))?;

  if is_custom_catalog_format(&root) {
    let file: OmniGenCatalogCustomFile = serde_json::from_value(root)
      .with_context(|| format!("parsing custom catalog in {}", path.display()))?;

    let has_provider_table = file
      .providers
      .as_ref()
      .map(|providers| !providers.is_empty())
      .unwrap_or(false);

    let custom_entries = if has_provider_table {
      parse_custom_image_entries_with_providers(&file.image)?
    } else {
      parse_custom_image_entries_legacy(&file.image)?
    };

    let custom_image_registry = if has_provider_table {
      let provider_defs = parse_custom_image_providers(file.providers.as_ref().unwrap())?;
      Some(build_custom_image_registry(&provider_defs, &custom_entries)?)
    } else {
      merge_custom_image_catalog_into_self_hosted(&mut self_hosted_config, &custom_entries)?;
      None
    };

    let image = build_custom_image_models_response(&custom_entries);
    info!(
      "Loaded {} custom image model(s) from {} (version {}, {} provider backend(s))",
      custom_entries.len(),
      path.display(),
      file.version.unwrap_or(if has_provider_table { 3 } else { 2 }),
      file.providers.as_ref().map(|p| p.len()).unwrap_or(1),
    );

    let video = apply_optional_modality_allowlist(&base_video, &file.video, "video", apply_video_allowlist);
    let audio = apply_optional_modality_allowlist(&base_audio, &file.audio, "audio", apply_audio_allowlist);
    let mesh = apply_optional_modality_allowlist(&base_mesh, &file.mesh, "mesh", apply_mesh_allowlist);
    let splat = apply_optional_modality_allowlist(&base_splat, &file.splat, "splat", apply_splat_allowlist);

    return Ok((OmniGenCatalog {
      image,
      video,
      audio,
      mesh,
      splat,
      custom_image_registry,
    }, self_hosted_config));
  }

  let file: OmniGenCatalogAllowlistFile = serde_json::from_value(root)
    .with_context(|| format!("parsing allowlist catalog in {}", path.display()))?;

  info!(
    "Loaded omni-gen catalog allowlists from {} (hosted models only; use version 2 + objects for a fully custom image list)",
    path.display()
  );

  let allowlists = CatalogAllowlists {
    image: file.image.map(|ids| allowlist_from_model_ids(&ids, "image")),
    video: file.video.map(|ids| allowlist_from_model_ids(&ids, "video")),
    audio: file.audio.map(|ids| allowlist_from_model_ids(&ids, "audio")),
    mesh: file.mesh.map(|ids| allowlist_from_model_ids(&ids, "mesh")),
    splat: file.splat.map(|ids| allowlist_from_model_ids(&ids, "splat")),
  };

  Ok((OmniGenCatalog {
    image: apply_image_allowlist(&base_image, allowlists.image.as_ref()),
    video: apply_video_allowlist(&base_video, allowlists.video.as_ref()),
    audio: apply_audio_allowlist(&base_audio, allowlists.audio.as_ref()),
    mesh: apply_mesh_allowlist(&base_mesh, allowlists.mesh.as_ref()),
    splat: apply_splat_allowlist(&base_splat, allowlists.splat.as_ref()),
    custom_image_registry: None,
  }, self_hosted_config))
}

struct CatalogAllowlists {
  image: Option<HashSet<String>>,
  video: Option<HashSet<String>>,
  audio: Option<HashSet<String>>,
  mesh: Option<HashSet<String>>,
  splat: Option<HashSet<String>>,
}

fn is_custom_catalog_format(root: &Value) -> bool {
  if matches!(
    root.get("version").and_then(|v| v.as_u64()),
    Some(2) | Some(3)
  ) {
    return true;
  }
  root
    .get("image")
    .and_then(|image| image.as_array())
    .and_then(|items| items.first())
    .map(|first| first.is_object())
    .unwrap_or(false)
}

fn parse_custom_image_entries_legacy(
  entries: &[CustomImageCatalogEntryFile],
) -> AnyhowResult<Vec<CustomImageCatalogEntry>> {
  parse_custom_image_entries_inner(entries, false)
}

fn parse_custom_image_entries_with_providers(
  entries: &[CustomImageCatalogEntryFile],
) -> AnyhowResult<Vec<CustomImageCatalogEntry>> {
  parse_custom_image_entries_inner(entries, true)
}

fn parse_custom_image_entries_inner(
  entries: &[CustomImageCatalogEntryFile],
  require_provider: bool,
) -> AnyhowResult<Vec<CustomImageCatalogEntry>> {
  if entries.is_empty() {
    return Err(anyhow!("custom image catalog must include at least one model"));
  }

  let mut parsed = Vec::with_capacity(entries.len());
  let mut seen_ids = HashSet::new();

  for entry in entries {
    let id = entry.id.trim();
    if id.is_empty() {
      return Err(anyhow!("custom image catalog entry is missing 'id'"));
    }
    if !seen_ids.insert(id.to_string()) {
      return Err(anyhow!("duplicate custom image model id '{}'", id));
    }

    let provider_id = entry
      .provider
      .as_ref()
      .map(|name| name.trim())
      .filter(|name| !name.is_empty())
      .map(|name| name.to_string());

    let provider_id = match (require_provider, provider_id) {
      (true, None) => {
        return Err(anyhow!(
          "image model '{}' must include 'provider' when the catalog defines a providers table",
          id
        ));
      }
      (true, Some(provider_id)) => provider_id,
      (false, Some(provider_id)) => provider_id,
      (false, None) => "legacy_internal".to_string(),
    };

    let full_name = entry
      .full_name
      .as_ref()
      .map(|name| name.trim())
      .filter(|name| !name.is_empty())
      .map(|name| name.to_string())
      .unwrap_or_else(|| id.to_string());

    let provider_model = entry
      .provider_model
      .as_ref()
      .map(|name| name.trim())
      .filter(|name| !name.is_empty())
      .map(|name| name.to_string())
      .unwrap_or_else(|| id.to_string());

    parsed.push(CustomImageCatalogEntry {
      id: id.to_string(),
      full_name,
      provider_id,
      provider_model,
    });
  }

  Ok(parsed)
}

fn parse_custom_image_providers(
  providers: &[CustomImageProviderFile],
) -> AnyhowResult<Vec<CustomImageProviderDefinition>> {
  if providers.is_empty() {
    return Err(anyhow!("custom catalog providers table must not be empty"));
  }

  let mut parsed = Vec::with_capacity(providers.len());
  for provider in providers {
    let id = provider.id.trim();
    if id.is_empty() {
      return Err(anyhow!("custom provider entry is missing 'id'"));
    }

    let provider_type = provider.r#type.trim();
    if provider_type != "openai_compatible" {
      return Err(anyhow!(
        "provider '{}' has unsupported type '{}' (only openai_compatible is supported today)",
        id,
        provider_type
      ));
    }

    let base_url = provider.base_url.trim();
    if base_url.is_empty() {
      return Err(anyhow!("provider '{}' is missing base_url", id));
    }
    internal_provider_client::InternalProviderClient::validate_base_url(base_url)?;

    let api_key = resolve_provider_api_key(provider)?;
    let default_model = provider
      .default_model
      .as_ref()
      .map(|name| name.trim())
      .filter(|name| !name.is_empty())
      .map(|name| name.to_string())
      .unwrap_or_else(|| "default".to_string());

    parsed.push(CustomImageProviderDefinition {
      id: id.to_string(),
      base_url: base_url.to_string(),
      api_key,
      default_model,
    });
  }

  Ok(parsed)
}

fn resolve_provider_api_key(provider: &CustomImageProviderFile) -> AnyhowResult<String> {
  if let Some(inline_key) = provider
    .api_key
    .as_ref()
    .map(|value| value.trim())
    .filter(|value| !value.is_empty())
  {
    return Ok(inline_key.to_string());
  }

  if let Some(env_name) = provider
    .api_key_env
    .as_ref()
    .map(|value| value.trim())
    .filter(|value| !value.is_empty())
  {
    return provider_env_string(env_name);
  }

  Ok("unset".to_string())
}

pub fn merge_custom_image_catalog_into_self_hosted(
  self_hosted_config: &mut SelfHostedConfig,
  entries: &[CustomImageCatalogEntry],
) -> AnyhowResult<()> {
  let internal = self_hosted_config
    .internal_provider
    .as_mut()
    .ok_or_else(|| {
      anyhow!(
        "custom image catalog requires INTERNAL_PROVIDER_URL (OpenAI-compatible backend)"
      )
    })?;

  for entry in entries {
    if !internal.routed_image_models.iter().any(|id| id == &entry.id) {
      internal.routed_image_models.push(entry.id.clone());
    }
    internal
      .model_aliases
      .insert(entry.id.clone(), entry.provider_model.clone());
  }

  Ok(())
}

fn apply_optional_modality_allowlist<T, F>(
  base: &T,
  ids: &Option<Vec<String>>,
  label: &str,
  filter: F,
) -> T
where
  F: FnOnce(&T, Option<&HashSet<String>>) -> T,
  T: Clone,
{
  match ids {
    None => filter(base, Some(&HashSet::new())),
    Some(model_ids) => filter(base, Some(&allowlist_from_model_ids(model_ids, label))),
  }
}

fn apply_image_allowlist(
  base: &OmniGenImageModelsResponse,
  allowlist: Option<&HashSet<String>>,
) -> OmniGenImageModelsResponse {
  match allowlist {
    None => base.clone(),
    Some(ids) => {
      warn_unknown_catalog_ids(ids, &known_image_model_ids(base), "image");
      let filtered = filter_image_catalog(base, ids);
      info!(
        "OMNI_GEN image catalog: {} of {} models after allowlist",
        filtered.models.len(),
        base.models.len()
      );
      filtered
    }
  }
}

fn apply_video_allowlist(
  base: &OmniGenVideoModelsResponse,
  allowlist: Option<&HashSet<String>>,
) -> OmniGenVideoModelsResponse {
  match allowlist {
    None => base.clone(),
    Some(ids) => {
      warn_unknown_catalog_ids(ids, &known_video_model_ids(base), "video");
      let filtered = filter_video_catalog(base, ids);
      info!(
        "OMNI_GEN video catalog: {} of {} models after allowlist",
        filtered.models.len(),
        base.models.len()
      );
      filtered
    }
  }
}

fn apply_audio_allowlist(
  base: &OmniGenAudioModelsResponse,
  allowlist: Option<&HashSet<String>>,
) -> OmniGenAudioModelsResponse {
  match allowlist {
    None => base.clone(),
    Some(ids) => {
      warn_unknown_catalog_ids(ids, &known_audio_model_ids(base), "audio");
      let filtered = filter_audio_catalog(base, ids);
      info!(
        "OMNI_GEN audio catalog: {} of {} models after allowlist",
        filtered.models.len(),
        base.models.len()
      );
      filtered
    }
  }
}

fn apply_mesh_allowlist(
  base: &OmniGenMeshModelsResponse,
  allowlist: Option<&HashSet<String>>,
) -> OmniGenMeshModelsResponse {
  match allowlist {
    None => base.clone(),
    Some(ids) => {
      warn_unknown_catalog_ids(ids, &known_mesh_model_ids(base), "mesh");
      let filtered = filter_mesh_catalog(base, ids);
      info!(
        "OMNI_GEN mesh catalog: {} of {} models after allowlist",
        filtered.models.len(),
        base.models.len()
      );
      filtered
    }
  }
}

fn apply_splat_allowlist(
  base: &OmniGenSplatModelsResponse,
  allowlist: Option<&HashSet<String>>,
) -> OmniGenSplatModelsResponse {
  match allowlist {
    None => base.clone(),
    Some(ids) => {
      warn_unknown_catalog_ids(ids, &known_splat_model_ids(base), "splat");
      let filtered = filter_splat_catalog(base, ids);
      info!(
        "OMNI_GEN splat catalog: {} of {} models after allowlist",
        filtered.models.len(),
        base.models.len()
      );
      filtered
    }
  }
}
