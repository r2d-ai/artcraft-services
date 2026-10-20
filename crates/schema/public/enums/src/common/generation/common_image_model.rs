use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use utoipa::ToSchema;

#[cfg(test)]
use strum::EnumCount;
#[cfg(test)]
use strum::EnumIter;

/// Image models available for generation.
/// Mirrors artcraft_router::api::router_image_model::RouterImageModel.
///
/// Unknown API strings deserialize as [`Custom`] so self-host catalogs can use
/// arbitrary model ids without extending this enum.
#[derive(Clone, Debug, PartialEq, Eq, Hash, ToSchema)]
#[cfg_attr(test, derive(EnumIter, EnumCount))]
pub enum CommonImageModel {
  #[serde(rename = "flux_1_dev")]
  Flux1Dev,
  #[serde(rename = "flux_1_schnell")]
  Flux1Schnell,
  #[serde(rename = "flux_pro_1p1")]
  FluxPro11,
  #[serde(rename = "flux_pro_1p1_ultra")]
  FluxPro11Ultra,
  #[serde(rename = "gpt_image_1")]
  GptImage1,
  #[serde(rename = "gpt_image_1p5")]
  GptImage1p5,
  #[serde(rename = "gpt_image_2")]
  GptImage2,
  #[serde(rename = "gpt_image_2p5_flare")]
  GptImage2p5Flare,
  #[serde(rename = "gpt_image_2p5_sunburst")]
  GptImage2p5Sunburst,
  #[serde(rename = "grok_imagine_image")]
  GrokImagineImage,
  #[serde(rename = "grok_imagine_image_q")]
  GrokImagineImageQuality,
  #[serde(rename = "midjourney_7")]
  Midjourney7,
  #[serde(rename = "midjourney_7_niji")]
  Midjourney7Niji,
  #[serde(rename = "midjourney_8")]
  Midjourney8,
  #[serde(rename = "nano_banana")]
  NanoBanana,
  #[serde(rename = "nano_banana_2")]
  NanoBanana2,
  #[serde(rename = "nano_banana_pro")]
  NanoBananaPro,
  #[serde(rename = "seedream_4")]
  Seedream4,
  #[serde(rename = "seedream_4p5")]
  Seedream4p5,
  #[serde(rename = "seedream_5_lite")]
  Seedream5Lite,
  #[serde(rename = "seedream_5p0_pro")]
  Seedream5p0Pro,
  #[serde(rename = "seedream_5p0_pro_u")]
  Seedream5p0ProUltra,
  #[serde(rename = "qwen_edit_2511_angles")]
  QwenEdit2511Angles,
  #[serde(rename = "flux_2_lora_angles")]
  Flux2LoraAngles,
  /// Self-host deployments: an image model served by the internal
  /// (OpenAI-compatible) provider. See `INTERNAL_PROVIDER_ROUTED_IMAGE_MODELS`.
  #[serde(rename = "minimax_image")]
  MinimaxImage,
  /// Self-host file catalog: arbitrary model id not in the hosted enum set.
  #[cfg_attr(test, strum(disabled))]
  Custom(String),
}

impl Serialize for CommonImageModel {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&self.api_id())
  }
}

impl<'de> Deserialize<'de> for CommonImageModel {
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    let id = String::deserialize(deserializer)?;
    if id.trim().is_empty() {
      return Err(D::Error::custom("image model id must not be empty"));
    }
    Ok(Self::from_api_id(id))
  }
}

impl CommonImageModel {
  pub fn api_id(&self) -> String {
    match self {
      Self::Flux1Dev => "flux_1_dev".to_string(),
      Self::Flux1Schnell => "flux_1_schnell".to_string(),
      Self::FluxPro11 => "flux_pro_1p1".to_string(),
      Self::FluxPro11Ultra => "flux_pro_1p1_ultra".to_string(),
      Self::GptImage1 => "gpt_image_1".to_string(),
      Self::GptImage1p5 => "gpt_image_1p5".to_string(),
      Self::GptImage2 => "gpt_image_2".to_string(),
      Self::GptImage2p5Flare => "gpt_image_2p5_flare".to_string(),
      Self::GptImage2p5Sunburst => "gpt_image_2p5_sunburst".to_string(),
      Self::GrokImagineImage => "grok_imagine_image".to_string(),
      Self::GrokImagineImageQuality => "grok_imagine_image_q".to_string(),
      Self::Midjourney7 => "midjourney_7".to_string(),
      Self::Midjourney7Niji => "midjourney_7_niji".to_string(),
      Self::Midjourney8 => "midjourney_8".to_string(),
      Self::NanoBanana => "nano_banana".to_string(),
      Self::NanoBanana2 => "nano_banana_2".to_string(),
      Self::NanoBananaPro => "nano_banana_pro".to_string(),
      Self::Seedream4 => "seedream_4".to_string(),
      Self::Seedream4p5 => "seedream_4p5".to_string(),
      Self::Seedream5Lite => "seedream_5_lite".to_string(),
      Self::Seedream5p0Pro => "seedream_5p0_pro".to_string(),
      Self::Seedream5p0ProUltra => "seedream_5p0_pro_u".to_string(),
      Self::QwenEdit2511Angles => "qwen_edit_2511_angles".to_string(),
      Self::Flux2LoraAngles => "flux_2_lora_angles".to_string(),
      Self::MinimaxImage => "minimax_image".to_string(),
      Self::Custom(id) => id.clone(),
    }
  }

  pub fn from_api_id(id: String) -> Self {
    match id.as_str() {
      "flux_1_dev" => Self::Flux1Dev,
      "flux_1_schnell" => Self::Flux1Schnell,
      "flux_pro_1p1" => Self::FluxPro11,
      "flux_pro_1p1_ultra" => Self::FluxPro11Ultra,
      "gpt_image_1" => Self::GptImage1,
      "gpt_image_1p5" => Self::GptImage1p5,
      "gpt_image_2" => Self::GptImage2,
      "gpt_image_2p5_flare" => Self::GptImage2p5Flare,
      "gpt_image_2p5_sunburst" => Self::GptImage2p5Sunburst,
      "grok_imagine_image" => Self::GrokImagineImage,
      "grok_imagine_image_q" => Self::GrokImagineImageQuality,
      "midjourney_7" => Self::Midjourney7,
      "midjourney_7_niji" => Self::Midjourney7Niji,
      "midjourney_8" => Self::Midjourney8,
      "nano_banana" => Self::NanoBanana,
      "nano_banana_2" => Self::NanoBanana2,
      "nano_banana_pro" => Self::NanoBananaPro,
      "seedream_4" => Self::Seedream4,
      "seedream_4p5" => Self::Seedream4p5,
      "seedream_5_lite" => Self::Seedream5Lite,
      "seedream_5p0_pro" => Self::Seedream5p0Pro,
      "seedream_5p0_pro_u" => Self::Seedream5p0ProUltra,
      "qwen_edit_2511_angles" => Self::QwenEdit2511Angles,
      "flux_2_lora_angles" => Self::Flux2LoraAngles,
      "minimax_image" => Self::MinimaxImage,
      other => Self::Custom(other.to_string()),
    }
  }

  pub fn is_custom_catalog_id(&self) -> bool {
    matches!(self, Self::Custom(_))
  }

  pub fn to_common_model_type(&self) -> crate::common::generation::common_model_type::CommonModelType {
    use crate::common::generation::common_model_type::CommonModelType;
    match self {
      Self::Flux1Dev => CommonModelType::Flux1Dev,
      Self::Flux1Schnell => CommonModelType::Flux1Schnell,
      Self::FluxPro11 => CommonModelType::FluxPro11,
      Self::FluxPro11Ultra => CommonModelType::FluxPro11Ultra,
      Self::GptImage1 => CommonModelType::GptImage1,
      Self::GptImage1p5 => CommonModelType::GptImage1p5,
      Self::GptImage2 => CommonModelType::GptImage2,
      Self::GptImage2p5Flare => CommonModelType::GptImage2p5Flare,
      Self::GptImage2p5Sunburst => CommonModelType::GptImage2p5Sunburst,
      Self::GrokImagineImage => CommonModelType::GrokImagineImage,
      Self::GrokImagineImageQuality => CommonModelType::GrokImagineImageQuality,
      Self::Midjourney7 => CommonModelType::Midjourney7,
      Self::Midjourney7Niji => CommonModelType::Midjourney7Niji,
      Self::Midjourney8 => CommonModelType::Midjourney8,
      Self::NanoBanana => CommonModelType::NanoBanana,
      Self::NanoBanana2 => CommonModelType::NanoBanana2,
      Self::NanoBananaPro => CommonModelType::NanoBananaPro,
      Self::Seedream4 => CommonModelType::Seedream4,
      Self::Seedream4p5 => CommonModelType::Seedream4p5,
      Self::Seedream5Lite => CommonModelType::Seedream5Lite,
      Self::Seedream5p0Pro => CommonModelType::Seedream5p0Pro,
      Self::Seedream5p0ProUltra => CommonModelType::Seedream5p0ProUltra,
      Self::QwenEdit2511Angles => CommonModelType::QwenEdit2511Angles,
      Self::Flux2LoraAngles => CommonModelType::Flux2LoraAngles,
      Self::MinimaxImage | Self::Custom(_) => CommonModelType::MinimaxImage,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::common::generation::common_model_class::CommonModelClass;
  use crate::common::generation::common_model_type::CommonModelType;
  use crate::test_helpers::assert_serialization;

  mod explicit_checks {
    use super::*;

    #[test]
    fn serialization() {
      assert_serialization(CommonImageModel::Flux1Dev, "flux_1_dev");
      assert_serialization(CommonImageModel::Flux1Schnell, "flux_1_schnell");
      assert_serialization(CommonImageModel::FluxPro11, "flux_pro_1p1");
      assert_serialization(CommonImageModel::FluxPro11Ultra, "flux_pro_1p1_ultra");
      assert_serialization(CommonImageModel::GptImage1, "gpt_image_1");
      assert_serialization(CommonImageModel::GptImage1p5, "gpt_image_1p5");
      assert_serialization(CommonImageModel::GptImage2, "gpt_image_2");
      assert_serialization(CommonImageModel::GptImage2p5Flare, "gpt_image_2p5_flare");
      assert_serialization(CommonImageModel::GptImage2p5Sunburst, "gpt_image_2p5_sunburst");
      assert_serialization(CommonImageModel::GrokImagineImage, "grok_imagine_image");
      assert_serialization(CommonImageModel::GrokImagineImageQuality, "grok_imagine_image_q");
      assert_serialization(CommonImageModel::Midjourney7, "midjourney_7");
      assert_serialization(CommonImageModel::Midjourney7Niji, "midjourney_7_niji");
      assert_serialization(CommonImageModel::Midjourney8, "midjourney_8");
      assert_serialization(CommonImageModel::NanoBanana, "nano_banana");
      assert_serialization(CommonImageModel::NanoBanana2, "nano_banana_2");
      assert_serialization(CommonImageModel::NanoBananaPro, "nano_banana_pro");
      assert_serialization(CommonImageModel::Seedream4, "seedream_4");
      assert_serialization(CommonImageModel::Seedream4p5, "seedream_4p5");
      assert_serialization(CommonImageModel::Seedream5Lite, "seedream_5_lite");
      assert_serialization(CommonImageModel::Seedream5p0Pro, "seedream_5p0_pro");
      assert_serialization(CommonImageModel::Seedream5p0ProUltra, "seedream_5p0_pro_u");
      assert_serialization(CommonImageModel::QwenEdit2511Angles, "qwen_edit_2511_angles");
      assert_serialization(CommonImageModel::Flux2LoraAngles, "flux_2_lora_angles");
    }

    #[test]
    fn custom_model_id_round_trip() {
      assert_serialization(
        CommonImageModel::Custom("company_logo_v1".to_string()),
        "company_logo_v1",
      );
    }

    #[test]
    fn all_image_models_convert_to_common_model_type() {
      let models = [
        (CommonImageModel::Flux1Dev, CommonModelType::Flux1Dev),
        (CommonImageModel::Flux1Schnell, CommonModelType::Flux1Schnell),
        (CommonImageModel::FluxPro11, CommonModelType::FluxPro11),
        (CommonImageModel::FluxPro11Ultra, CommonModelType::FluxPro11Ultra),
        (CommonImageModel::GptImage1, CommonModelType::GptImage1),
        (CommonImageModel::GptImage1p5, CommonModelType::GptImage1p5),
        (CommonImageModel::GptImage2, CommonModelType::GptImage2),
        (CommonImageModel::GptImage2p5Flare, CommonModelType::GptImage2p5Flare),
        (CommonImageModel::GptImage2p5Sunburst, CommonModelType::GptImage2p5Sunburst),
        (CommonImageModel::GrokImagineImage, CommonModelType::GrokImagineImage),
        (CommonImageModel::GrokImagineImageQuality, CommonModelType::GrokImagineImageQuality),
        (CommonImageModel::Midjourney7, CommonModelType::Midjourney7),
        (CommonImageModel::Midjourney7Niji, CommonModelType::Midjourney7Niji),
        (CommonImageModel::Midjourney8, CommonModelType::Midjourney8),
        (CommonImageModel::NanoBanana, CommonModelType::NanoBanana),
        (CommonImageModel::NanoBanana2, CommonModelType::NanoBanana2),
        (CommonImageModel::NanoBananaPro, CommonModelType::NanoBananaPro),
        (CommonImageModel::Seedream4, CommonModelType::Seedream4),
        (CommonImageModel::Seedream4p5, CommonModelType::Seedream4p5),
        (CommonImageModel::Seedream5Lite, CommonModelType::Seedream5Lite),
        (CommonImageModel::Seedream5p0Pro, CommonModelType::Seedream5p0Pro),
        (CommonImageModel::Seedream5p0ProUltra, CommonModelType::Seedream5p0ProUltra),
        (CommonImageModel::QwenEdit2511Angles, CommonModelType::QwenEdit2511Angles),
        (CommonImageModel::Flux2LoraAngles, CommonModelType::Flux2LoraAngles),
      ];
      for (image_model, expected) in models {
        assert_eq!(image_model.to_common_model_type(), expected);
      }
    }

    #[test]
    fn all_image_models_map_to_image_class() {
      use strum::IntoEnumIterator;
      for variant in CommonImageModel::iter() {
        let model_type = variant.to_common_model_type();
        assert_eq!(
          model_type.get_model_class(), CommonModelClass::Image,
          "{:?} mapped to {:?} which has class {:?}, expected Image",
          variant, model_type, model_type.get_model_class(),
        );
      }
    }
  }

  mod mechanical_checks {
    use super::*;

    #[test]
    fn variant_count() {
      use strum::IntoEnumIterator;
      assert_eq!(CommonImageModel::iter().len(), 24);
    }

    #[test]
    fn serde_round_trip() {
      use strum::IntoEnumIterator;
      for variant in CommonImageModel::iter() {
        let json = serde_json::to_string(&variant).unwrap();
        let deserialized: CommonImageModel = serde_json::from_str(&json).unwrap();
        assert_eq!(variant, deserialized, "round-trip failed for {:?}", variant);
      }
    }

    #[test]
    fn serialized_names_are_lowercase_alphanumeric_underscore() {
      use strum::IntoEnumIterator;
      let valid = regex::Regex::new(r"^[a-z0-9_]+$").unwrap();
      for variant in CommonImageModel::iter() {
        let json = serde_json::to_string(&variant).unwrap().replace('"', "");
        assert!(
          valid.is_match(&json),
          "{:?} serializes to {:?} which contains invalid characters",
          variant, json,
        );
      }
    }

    #[test]
    fn to_common_model_type_covers_all_variants() {
      use strum::IntoEnumIterator;
      for variant in CommonImageModel::iter() {
        let _ = variant.to_common_model_type();
      }
    }
  }
}
