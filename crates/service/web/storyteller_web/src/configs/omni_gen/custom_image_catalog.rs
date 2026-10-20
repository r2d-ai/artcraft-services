use artcraft_api_defs::omni_gen::models::omni_gen_image_models::{
  OmniGenImageModelDetails, OmniGenImageModelProviderDetails, OmniGenImageModelsResponse,
  OmniGenImageProviderModelDetails,
};
use enums::common::generation::common_aspect_ratio::CommonAspectRatio;
use enums::common::generation::common_image_model::CommonImageModel;
use enums::common::generation_provider::GenerationProvider;

#[derive(Clone, Debug)]
pub struct CustomImageCatalogEntry {
  pub id: String,
  pub full_name: String,
  pub provider_id: String,
  pub provider_model: String,
}

pub fn build_custom_image_models_response(
  entries: &[CustomImageCatalogEntry],
) -> OmniGenImageModelsResponse {
  let models: Vec<OmniGenImageModelDetails> = entries
    .iter()
    .map(|entry| default_image_model_details(entry))
    .collect();

  let mut providers_by_id: std::collections::BTreeMap<String, Vec<OmniGenImageProviderModelDetails>> =
    std::collections::BTreeMap::new();
  for entry in entries {
    providers_by_id
      .entry(entry.provider_id.clone())
      .or_default()
      .push(OmniGenImageProviderModelDetails {
        model: CommonImageModel::from_api_id(entry.id.clone()),
        overrides: None,
      });
  }

  let providers: Vec<OmniGenImageModelProviderDetails> = providers_by_id
    .into_iter()
    .map(|(_provider_id, models)| OmniGenImageModelProviderDetails {
      provider: GenerationProvider::Artcraft,
      models,
    })
    .collect();

  OmniGenImageModelsResponse {
    success: true,
    models,
    providers,
  }
}

fn default_image_model_details(entry: &CustomImageCatalogEntry) -> OmniGenImageModelDetails {
  OmniGenImageModelDetails {
    model: CommonImageModel::from_api_id(entry.id.clone()),
    model_creator: None,
    full_name: Some(entry.full_name.clone()),
    text_prompt_supported: Some(true),
    aspect_ratio_options: Some(vec![
      CommonAspectRatio::SquareHd,
      CommonAspectRatio::Square,
      CommonAspectRatio::TallThreeByFour,
      CommonAspectRatio::TallNineBySixteen,
      CommonAspectRatio::WideFourByThree,
      CommonAspectRatio::WideSixteenByNine,
    ]),
    aspect_ratio_default: Some(CommonAspectRatio::Square),
    batch_size_min: Some(1),
    batch_size_max: Some(4),
    batch_size_default: Some(1),
    ..Default::default()
  }
}
