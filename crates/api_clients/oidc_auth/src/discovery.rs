use errors::AnyhowResult;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct OidcProviderMetadata {
  pub issuer: String,
  pub authorization_endpoint: String,
  pub token_endpoint: String,
  pub jwks_uri: String,
}

pub async fn fetch_provider_metadata(issuer: &str) -> AnyhowResult<OidcProviderMetadata> {
  let issuer = issuer.trim_end_matches('/');
  let discovery_url = format!("{issuer}/.well-known/openid-configuration");
  let response = reqwest::get(&discovery_url).await?;
  let metadata = response.json::<OidcProviderMetadata>().await?;
  Ok(metadata)
}
