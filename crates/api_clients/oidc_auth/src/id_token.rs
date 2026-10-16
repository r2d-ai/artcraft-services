use std::collections::HashSet;

use anyhow::anyhow;
use jwt_simple::algorithms::RS256PublicKey;
use jwt_simple::algorithms::RSAPublicKeyLike;
use jwt_simple::prelude::VerificationOptions;
use serde::Deserialize;
use serde::Serialize;

use jwt_simple::claims::Audiences;

use crate::jwt_header::decode_jwt_header;
use crate::jwks::{jwk_set_to_key_map, KeyMap};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OidcCustomClaims {
  pub email: Option<String>,
  pub email_verified: Option<bool>,
  pub name: Option<String>,
  pub given_name: Option<String>,
  pub family_name: Option<String>,
  pub locale: Option<String>,
  pub groups: Option<Vec<String>>,
  pub nonce: Option<String>,
}

#[derive(Clone, Debug)]
pub struct OidcIdTokenClaims {
  pub issuer: String,
  pub subject: String,
  pub email: Option<String>,
  pub email_verified: bool,
  pub name: Option<String>,
  pub given_name: Option<String>,
  pub family_name: Option<String>,
  pub locale: Option<String>,
  pub groups: Vec<String>,
}

pub async fn fetch_jwks_key_map(jwks_uri: &str) -> anyhow::Result<KeyMap> {
  let body = reqwest::get(jwks_uri).await?.text().await?;
  jwk_set_to_key_map(&body)
}

pub fn verify_id_token(
  keys: &KeyMap,
  id_token: &str,
  issuer: &str,
  client_id: &str,
  expected_nonce: Option<&str>,
) -> anyhow::Result<OidcIdTokenClaims> {
  let header_kid = decode_jwt_header(id_token)?
    .kid
    .ok_or_else(|| anyhow!("JWT missing kid"))?;

  let key = keys
    .get(&header_kid)
    .or_else(|| keys.values().next())
    .ok_or_else(|| anyhow!("JWKS missing key {}", header_kid))?;

  verify_with_key(key, id_token, issuer, client_id, expected_nonce)
}

fn verify_with_key(
  key: &RS256PublicKey,
  id_token: &str,
  issuer: &str,
  client_id: &str,
  expected_nonce: Option<&str>,
) -> anyhow::Result<OidcIdTokenClaims> {
  let issuer_trimmed = issuer.trim_end_matches('/');
  let options = VerificationOptions {
    allowed_issuers: Some(HashSet::from([
      issuer.to_string(),
      issuer_trimmed.to_string(),
    ])),
    ..Default::default()
  };

  let token = key.verify_token::<OidcCustomClaims>(id_token, Some(options))?;

  if !audience_matches(&token.audiences, client_id) {
    return Err(anyhow!("OIDC audience mismatch"));
  }

  if let Some(expected) = expected_nonce {
    if token.custom.nonce.as_deref() != Some(expected) {
      return Err(anyhow!("OIDC nonce mismatch"));
    }
  }

  Ok(OidcIdTokenClaims {
    issuer: token.issuer.unwrap_or_default(),
    subject: token.subject.unwrap_or_default(),
    email: token.custom.email,
    email_verified: token.custom.email_verified.unwrap_or(false),
    name: token.custom.name,
    given_name: token.custom.given_name,
    family_name: token.custom.family_name,
    locale: token.custom.locale,
    groups: token.custom.groups.unwrap_or_default(),
  })
}

fn audience_matches(audiences: &Option<Audiences>, client_id: &str) -> bool {
  let allowed = HashSet::from([client_id.to_string()]);
  match audiences {
    Some(audiences) => audiences.contains(&allowed),
    None => false,
  }
}
