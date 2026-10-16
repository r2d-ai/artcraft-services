use anyhow::anyhow;
use errors::AnyhowResult;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
  pub access_token: String,
  pub id_token: String,
  pub token_type: String,
  pub expires_in: Option<u64>,
}

pub async fn exchange_authorization_code(
  token_endpoint: &str,
  client_id: &str,
  client_secret: Option<&str>,
  redirect_uri: &str,
  code: &str,
  code_verifier: &str,
) -> AnyhowResult<TokenResponse> {
  let mut form = vec![
    ("grant_type", "authorization_code"),
    ("code", code),
    ("redirect_uri", redirect_uri),
    ("client_id", client_id),
    ("code_verifier", code_verifier),
  ];

  let client = reqwest::Client::new();
  let mut request = client.post(token_endpoint).form(&form);

  if let Some(secret) = client_secret {
    request = request.basic_auth(client_id, Some(secret));
  }

  let response = request.send().await?;
  if !response.status().is_success() {
    let body = response.text().await.unwrap_or_default();
    return Err(anyhow::anyhow!("OIDC token exchange failed: {}", body));
  }

  Ok(response.json::<TokenResponse>().await?)
}
