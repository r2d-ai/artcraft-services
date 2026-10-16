use std::sync::Arc;

use actix_artcraft::sessions::user_sessions::http_user_session_manager::HttpUserSessionManager;
use actix_web::{web, HttpRequest, HttpResponse};
use http_server_common::request::get_request_ip::get_request_ip;
use log::{info, warn};
use mysql_queries::queries::oidc_sign_in_accounts::get_oidc_sign_in_account_by_issuer_subject::{
  get_oidc_sign_in_account_by_issuer_subject, GetOidcSignInAccountByIssuerSubjectArgs,
};
use mysql_queries::queries::oidc_sign_in_accounts::insert_oidc_sign_in_account::{
  insert_oidc_sign_in_account, InsertOidcSignInAccountArgs,
};
use mysql_queries::queries::users::user::create::create_account_from_google_sso::{
  create_account_from_google_sso, CreateAccountFromGoogleSsoArgs,
};
use mysql_queries::queries::users::user_sessions::create_user_session_with_transactor::create_user_session_with_transactor;
use mysql_queries::utils::transactor::Transactor;
use oidc_auth::id_token::{fetch_jwks_key_map, verify_id_token};
use oidc_auth::token_exchange::exchange_authorization_code;
use redis_common::redis_keys::RedisKeys;
use sqlx::Acquire;
use tokens::tokens::oidc_sign_in_accounts::OidcSignInAccountToken;
use tokens::tokens::users::UserToken;
use users::email::email_to_gravatar_hash::email_to_gravatar_hash;
use users::username::generate_random_username::generate_random_username;

use crate::http_server::common_responses::common_web_error::CommonWebError;
use crate::state::server_state::ServerState;

#[derive(serde::Deserialize)]
pub struct OidcCallbackQuery {
  pub code: Option<String>,
  pub state: Option<String>,
  pub error: Option<String>,
}

pub async fn oidc_callback_handler(
  http_request: HttpRequest,
  query: web::Query<OidcCallbackQuery>,
  server_state: web::Data<Arc<ServerState>>,
  session_cookie_manager: web::Data<HttpUserSessionManager>,
) -> Result<HttpResponse, CommonWebError> {
  if let Some(error) = &query.error {
    warn!("OIDC provider returned error: {}", error);
    return Err(CommonWebError::BadInputWithSimpleMessage(error.clone()));
  }

  let code = query
    .code
    .as_deref()
    .ok_or_else(|| CommonWebError::BadInputWithSimpleMessage("missing code".to_string()))?;
  let state = query
    .state
    .as_deref()
    .ok_or_else(|| CommonWebError::BadInputWithSimpleMessage("missing state".to_string()))?;

  let oidc = server_state
    .oidc_config
    .as_ref()
    .ok_or_else(|| CommonWebError::NotFound)?;

  let redis_key = RedisKeys::oidc_auth_state(state);
  let mut redis_connection = server_state.redis_pool.get().map_err(CommonWebError::from_error)?;
  let stored: Option<String> = redis::cmd("GET")
    .arg(&redis_key)
    .query(&mut *redis_connection)
    .map_err(CommonWebError::from_error)?;
  let stored = stored.ok_or(CommonWebError::NotAuthorized)?;
  redis::cmd("DEL")
    .arg(&redis_key)
    .query::<()>(&mut *redis_connection)
    .map_err(CommonWebError::from_error)?;

  let mut parts = stored.splitn(2, '\n').collect::<Vec<_>>();
  if parts.len() != 2 {
    return Err(CommonWebError::NotAuthorized);
  }
  let code_verifier = parts[0];
  let expected_nonce = parts[1];

  let token_response = exchange_authorization_code(
    &oidc.metadata.token_endpoint,
    &oidc.client_id,
    oidc.client_secret.as_deref(),
    &oidc.redirect_uri,
    code,
    code_verifier,
  )
    .await
    .map_err(CommonWebError::from_anyhow_error)?;

  let jwks = fetch_jwks_key_map(&oidc.metadata.jwks_uri)
    .await
    .map_err(CommonWebError::from_anyhow_error)?;

  let claims = verify_id_token(
    &jwks,
    &token_response.id_token,
    &oidc.issuer,
    &oidc.client_id,
    Some(expected_nonce),
  )
    .map_err(CommonWebError::from_anyhow_error)?;

  let mut mysql_connection = server_state.mysql_pool.acquire().await.map_err(CommonWebError::from_error)?;
  let ip_address = get_request_ip(&http_request);

  let existing = get_oidc_sign_in_account_by_issuer_subject(GetOidcSignInAccountByIssuerSubjectArgs {
    issuer: &oidc.issuer,
    subject: &claims.subject,
    mysql_executor: &mut *mysql_connection,
  })
    .await
    .map_err(CommonWebError::from_error)?;

  let user_token = if let Some(account) = existing {
    account
      .maybe_user_token
      .ok_or_else(|| CommonWebError::server_error_with_message("OIDC account is not linked to a user"))?
  } else {
    create_oidc_user_and_account(
      &oidc.issuer,
      &claims,
      &ip_address,
      &mut mysql_connection,
    )
      .await?
  };

  let session_token = create_user_session_with_transactor(
    &user_token,
    &ip_address,
    Transactor::for_connection(&mut *mysql_connection),
  )
    .await
    .map_err(CommonWebError::from_anyhow_error)?;

  let session_cookie = session_cookie_manager
    .create_cookie(&session_token, &user_token)
    .map_err(CommonWebError::from_error)?;

  info!("OIDC login succeeded for user {}", user_token);

  Ok(HttpResponse::Found()
    .append_header(("Location", oidc.post_login_redirect.clone()))
    .cookie(session_cookie)
    .finish())
}

async fn create_oidc_user_and_account(
  issuer: &str,
  claims: &oidc_auth::id_token::OidcIdTokenClaims,
  ip_address: &str,
  mysql_connection: &mut sqlx::pool::PoolConnection<sqlx::MySql>,
) -> Result<UserToken, CommonWebError> {
  let email = claims
    .email
    .clone()
    .ok_or_else(|| CommonWebError::BadInputWithSimpleMessage("email claim required".to_string()))?;

  let mut transaction = mysql_connection.begin().await.map_err(CommonWebError::from_error)?;

  let display_name = claims
    .name
    .clone()
    .unwrap_or_else(|| generate_random_username());
  let username = display_name.trim().to_lowercase();
  let email_gravatar_hash = email_to_gravatar_hash(&email);

  let user_token = create_account_from_google_sso(
    CreateAccountFromGoogleSsoArgs {
      username: &username,
      display_name: &display_name,
      email_address: &email,
      email_gravatar_hash: &email_gravatar_hash,
      email_confirmed_by_google: claims.email_verified,
      ip_address,
      maybe_source: None,
      maybe_feature_flags: None,
      maybe_referral_url: None,
      maybe_landing_url: None,
      maybe_referral_partner: None,
      maybe_referral_user_token: None,
    },
    &mut *transaction,
  )
    .await
    .map_err(|err| {
      warn!("OIDC account creation failed: {:?}", err);
      CommonWebError::BadInputWithSimpleMessage("could not create account".to_string())
    })?;

  let oidc_account_token = OidcSignInAccountToken::generate();
  insert_oidc_sign_in_account(InsertOidcSignInAccountArgs {
    token: &oidc_account_token,
    issuer,
    subject: &claims.subject,
    maybe_user_token: Some(&user_token),
    maybe_email_address: Some(&email),
    is_email_verified: claims.email_verified,
    maybe_name: claims.name.as_deref(),
    ip_address,
    mysql_executor: &mut *transaction,
  })
    .await
    .map_err(CommonWebError::from_error)?;

  transaction.commit().await.map_err(CommonWebError::from_error)?;
  Ok(user_token)
}
