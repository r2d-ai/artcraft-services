use errors::AnyhowResult;
use oidc_auth::discovery::OidcProviderMetadata;

#[derive(Clone, Debug)]
pub struct OidcConfig {
  pub enabled: bool,
  pub issuer: String,
  pub client_id: String,
  pub client_secret: Option<String>,
  pub redirect_uri: String,
  pub post_login_redirect: String,
  pub metadata: OidcProviderMetadata,
}

pub async fn setup_oidc_config() -> AnyhowResult<Option<OidcConfig>> {
  let enabled = easyenv::get_env_bool_or_default("OIDC_ENABLED", false);
  if !enabled {
    return Ok(None);
  }

  let issuer = easyenv::get_env_string_required("OIDC_ISSUER")?;
  let client_id = easyenv::get_env_string_required("OIDC_CLIENT_ID")?;
  let client_secret = easyenv::get_env_string_optional("OIDC_CLIENT_SECRET")
    .filter(|value| !value.is_empty());
  let redirect_uri = easyenv::get_env_string_required("OIDC_REDIRECT_URI")?;
  let post_login_redirect = easyenv::get_env_string_optional("OIDC_POST_LOGIN_REDIRECT")
    .unwrap_or_else(|| easyenv::get_env_string_or_default("WEBSITE_HOMEPAGE_REDIRECT", "/"));

  let metadata = oidc_auth::discovery::fetch_provider_metadata(&issuer).await?;

  Ok(Some(OidcConfig {
    enabled: true,
    issuer,
    client_id,
    client_secret,
    redirect_uri,
    post_login_redirect,
    metadata,
  }))
}
