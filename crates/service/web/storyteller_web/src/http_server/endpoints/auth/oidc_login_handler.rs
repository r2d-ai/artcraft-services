use std::sync::Arc;

use actix_web::{web, HttpResponse};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use log::info;
use oidc_auth::pkce::generate_pkce_pair;
use rand::{rngs::OsRng, TryRngCore};
use redis_common::redis_keys::RedisKeys;
use url::Url;

use crate::http_server::common_responses::common_web_error::CommonWebError;
use crate::state::server_state::ServerState;

const OIDC_STATE_TTL_SECONDS: usize = 600;

pub async fn oidc_login_handler(
  server_state: web::Data<Arc<ServerState>>,
) -> Result<HttpResponse, CommonWebError> {
  let oidc = server_state
    .oidc_config
    .as_ref()
    .ok_or_else(|| CommonWebError::NotFound)?;

  let pkce = generate_pkce_pair().map_err(CommonWebError::from_anyhow_error)?;
  let mut state_bytes = [0u8; 32];
  OsRng.try_fill_bytes(&mut state_bytes).map_err(CommonWebError::from_error)?;
  let state = URL_SAFE_NO_PAD.encode(state_bytes);
  let mut nonce_bytes = [0u8; 16];
  OsRng.try_fill_bytes(&mut nonce_bytes).map_err(CommonWebError::from_error)?;
  let nonce = URL_SAFE_NO_PAD.encode(nonce_bytes);

  let payload = format!("{}\n{}", pkce.code_verifier, nonce);
  let redis_key = RedisKeys::oidc_auth_state(&state);
  let mut redis_connection = server_state.redis_pool.get().map_err(CommonWebError::from_error)?;
  redis::cmd("SETEX")
    .arg(&redis_key)
    .arg(OIDC_STATE_TTL_SECONDS)
    .arg(payload)
    .query::<()>(&mut *redis_connection)
    .map_err(CommonWebError::from_error)?;

  let authorize_url = Url::parse_with_params(
    &oidc.metadata.authorization_endpoint,
    &[
      ("response_type", "code"),
      ("client_id", oidc.client_id.as_str()),
      ("redirect_uri", oidc.redirect_uri.as_str()),
      ("scope", "openid profile email"),
      ("state", state.as_str()),
      ("nonce", nonce.as_str()),
      ("code_challenge", pkce.code_challenge.as_str()),
      ("code_challenge_method", "S256"),
    ],
  )
    .map_err(CommonWebError::from_error)?;

  info!("Redirecting browser to OIDC provider");
  Ok(HttpResponse::Found()
    .append_header(("Location", authorize_url.to_string()))
    .finish())
}
