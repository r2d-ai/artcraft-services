use std::collections::HashSet;

use artcraft_api_defs::omni_gen::models::omni_gen_audio_models::OmniGenAudioModelsResponse;
use artcraft_api_defs::omni_gen::models::omni_gen_image_models::OmniGenImageModelsResponse;
use artcraft_api_defs::omni_gen::models::omni_gen_mesh_models::OmniGenMeshModelsResponse;
use artcraft_api_defs::omni_gen::models::omni_gen_splat_models::OmniGenSplatModelsResponse;
use artcraft_api_defs::omni_gen::models::omni_gen_video_models::OmniGenVideoModelsResponse;
use log::warn;
use serde::Serialize;

pub fn filter_image_catalog(
  response: &OmniGenImageModelsResponse,
  allowlist: &HashSet<String>,
) -> OmniGenImageModelsResponse {
  let mut filtered = response.clone();
  filtered
    .models
    .retain(|details| model_id_in_allowlist(&details.model, allowlist));
  for provider in &mut filtered.providers {
    provider
      .models
      .retain(|details| model_id_in_allowlist(&details.model, allowlist));
  }
  filtered.providers.retain(|provider| !provider.models.is_empty());
  filtered
}

pub fn filter_video_catalog(
  response: &OmniGenVideoModelsResponse,
  allowlist: &HashSet<String>,
) -> OmniGenVideoModelsResponse {
  let mut filtered = response.clone();
  filtered
    .models
    .retain(|details| model_id_in_allowlist(&details.model, allowlist));
  for provider in &mut filtered.providers {
    provider
      .models
      .retain(|details| model_id_in_allowlist(&details.model, allowlist));
  }
  filtered.providers.retain(|provider| !provider.models.is_empty());
  filtered
}

pub fn filter_audio_catalog(
  response: &OmniGenAudioModelsResponse,
  allowlist: &HashSet<String>,
) -> OmniGenAudioModelsResponse {
  let mut filtered = response.clone();
  filtered
    .models
    .retain(|details| model_id_in_allowlist(&details.model, allowlist));
  for provider in &mut filtered.providers {
    provider
      .models
      .retain(|details| model_id_in_allowlist(&details.model, allowlist));
  }
  filtered.providers.retain(|provider| !provider.models.is_empty());
  filtered
}

pub fn filter_mesh_catalog(
  response: &OmniGenMeshModelsResponse,
  allowlist: &HashSet<String>,
) -> OmniGenMeshModelsResponse {
  let mut filtered = response.clone();
  filtered
    .models
    .retain(|details| model_id_in_allowlist(&details.model, allowlist));
  for provider in &mut filtered.providers {
    provider
      .models
      .retain(|details| model_id_in_allowlist(&details.model, allowlist));
  }
  filtered.providers.retain(|provider| !provider.models.is_empty());
  filtered
}

pub fn filter_splat_catalog(
  response: &OmniGenSplatModelsResponse,
  allowlist: &HashSet<String>,
) -> OmniGenSplatModelsResponse {
  let mut filtered = response.clone();
  filtered
    .models
    .retain(|details| model_id_in_allowlist(&details.model, allowlist));
  for provider in &mut filtered.providers {
    provider
      .models
      .retain(|details| model_id_in_allowlist(&details.model, allowlist));
  }
  filtered.providers.retain(|provider| !provider.models.is_empty());
  filtered
}

pub fn allowlist_from_model_ids(model_ids: &[String], label: &str) -> HashSet<String> {
  let mut allowlist = HashSet::new();
  for model_id in model_ids {
    let trimmed = model_id.trim();
    if trimmed.is_empty() {
      continue;
    }
    if !allowlist.insert(trimmed.to_string()) {
      warn!(
        "OMNI_GEN catalog allowlist for {} contains duplicate model id '{}'",
        label,
        trimmed
      );
    }
  }
  allowlist
}

pub fn warn_unknown_catalog_ids(allowlist: &HashSet<String>, known_ids: &HashSet<String>, label: &str) {
  for model_id in allowlist {
    if !known_ids.contains(model_id) {
      warn!(
        "OMNI_GEN catalog allowlist for {} has unknown model id '{}' (not in built-in catalog); it will not appear in the API",
        label,
        model_id
      );
    }
  }
}

pub fn known_image_model_ids(response: &OmniGenImageModelsResponse) -> HashSet<String> {
  response
    .models
    .iter()
    .filter_map(|details| api_model_id(&details.model))
    .collect()
}

pub fn known_video_model_ids(response: &OmniGenVideoModelsResponse) -> HashSet<String> {
  response
    .models
    .iter()
    .filter_map(|details| api_model_id(&details.model))
    .collect()
}

pub fn known_audio_model_ids(response: &OmniGenAudioModelsResponse) -> HashSet<String> {
  response
    .models
    .iter()
    .filter_map(|details| api_model_id(&details.model))
    .collect()
}

pub fn known_mesh_model_ids(response: &OmniGenMeshModelsResponse) -> HashSet<String> {
  response
    .models
    .iter()
    .filter_map(|details| api_model_id(&details.model))
    .collect()
}

pub fn known_splat_model_ids(response: &OmniGenSplatModelsResponse) -> HashSet<String> {
  response
    .models
    .iter()
    .filter_map(|details| api_model_id(&details.model))
    .collect()
}

fn model_id_in_allowlist<T: Serialize>(model: &T, allowlist: &HashSet<String>) -> bool {
  match api_model_id(model) {
    Some(id) => allowlist.contains(&id),
    None => false,
  }
}

fn api_model_id<T: Serialize>(model: &T) -> Option<String> {
  serde_json::to_value(model)
    .ok()
    .and_then(|value| value.as_str().map(|s| s.to_string()))
}
