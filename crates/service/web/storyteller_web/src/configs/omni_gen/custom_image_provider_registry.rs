use std::collections::HashMap;

use internal_provider_client::InternalProviderClient;

use crate::configs::omni_gen::custom_image_catalog::CustomImageCatalogEntry;

#[derive(Clone, Debug)]
pub struct CustomImageRegistry {
  providers: HashMap<String, CustomImageProviderInstance>,
  routes: HashMap<String, CustomImageModelRoute>,
}

#[derive(Clone, Debug)]
pub struct CustomImageProviderInstance {
  pub id: String,
  pub backend: CustomImageProviderBackend,
}

#[derive(Clone, Debug)]
pub enum CustomImageProviderBackend {
  OpenAiCompatible(InternalProviderClient),
}

#[derive(Clone, Debug)]
pub struct CustomImageModelRoute {
  pub model_id: String,
  pub provider_id: String,
  pub provider_model: String,
}

impl CustomImageRegistry {
  pub fn route_for(&self, model_id: &str) -> Option<&CustomImageModelRoute> {
    self.routes.get(model_id)
  }

  pub fn provider_for_route(&self, route: &CustomImageModelRoute) -> Option<&CustomImageProviderInstance> {
    self.providers.get(&route.provider_id)
  }

  pub fn openai_client_for_model(&self, model_id: &str) -> Option<(&InternalProviderClient, String)> {
    let route = self.route_for(model_id)?;
    let provider = self.provider_for_route(route)?;
    match &provider.backend {
      CustomImageProviderBackend::OpenAiCompatible(client) => {
        Some((client, route.provider_model.clone()))
      }
    }
  }
}

pub fn build_custom_image_registry(
  provider_defs: &[CustomImageProviderDefinition],
  model_entries: &[CustomImageCatalogEntry],
) -> anyhow::Result<CustomImageRegistry> {
  let mut providers = HashMap::new();
  for def in provider_defs {
    if providers.contains_key(&def.id) {
      return Err(anyhow::anyhow!("duplicate custom provider id '{}'", def.id));
    }
    let backend = CustomImageProviderBackend::OpenAiCompatible(
      InternalProviderClient::new(def.base_url.clone(), def.api_key.clone(), def.default_model.clone()),
    );
    providers.insert(
      def.id.clone(),
      CustomImageProviderInstance {
        id: def.id.clone(),
        backend,
      },
    );
  }

  let mut routes = HashMap::new();
  for entry in model_entries {
    if routes.contains_key(&entry.id) {
      return Err(anyhow::anyhow!("duplicate custom image model id '{}'", entry.id));
    }
    if !providers.contains_key(&entry.provider_id) {
      return Err(anyhow::anyhow!(
        "image model '{}' references unknown provider '{}'",
        entry.id,
        entry.provider_id
      ));
    }
    routes.insert(
      entry.id.clone(),
      CustomImageModelRoute {
        model_id: entry.id.clone(),
        provider_id: entry.provider_id.clone(),
        provider_model: entry.provider_model.clone(),
      },
    );
  }

  if routes.is_empty() {
    return Err(anyhow::anyhow!("custom image catalog must include at least one model"));
  }

  Ok(CustomImageRegistry {
    providers,
    routes,
  })
}

#[derive(Clone, Debug)]
pub struct CustomImageProviderDefinition {
  pub id: String,
  pub base_url: String,
  pub api_key: String,
  pub default_model: String,
}
