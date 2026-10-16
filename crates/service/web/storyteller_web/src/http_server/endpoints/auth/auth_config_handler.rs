use std::sync::Arc;

use actix_web::{web, HttpRequest};
use serde::Serialize;

use crate::http_server::common_responses::common_web_error::CommonWebError;
use crate::state::server_state::ServerState;

#[derive(Serialize)]
pub struct AuthConfigResponse {
  pub success: bool,
  pub oidc_enabled: bool,
  pub oidc_login_path: Option<String>,
  pub oidc_only: bool,
  pub self_hosted: bool,
}

pub async fn auth_config_handler(
  _http_request: HttpRequest,
  server_state: web::Data<Arc<ServerState>>,
) -> Result<web::Json<AuthConfigResponse>, CommonWebError> {
  let oidc_enabled = server_state.oidc_config.is_some();
  Ok(web::Json(AuthConfigResponse {
    success: true,
    oidc_enabled,
    oidc_login_path: if oidc_enabled {
      Some("/v1/auth/oidc/login".to_string())
    } else {
      None
    },
    oidc_only: server_state.self_hosted_config.disable_public_signup,
    self_hosted: server_state.self_hosted_config.enabled,
  }))
}
