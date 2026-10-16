use anyhow::anyhow;
use jwt_simple::prelude::Base64UrlSafeNoPadding;
use jwt_simple::reexports::ct_codecs::Decoder;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct JwtHeader {
  pub kid: Option<String>,
}

pub fn decode_jwt_header(jwt: &str) -> anyhow::Result<JwtHeader> {
  let jwt_header_b64 = jwt
    .split('.')
    .next()
    .ok_or_else(|| anyhow!("JWT malformed"))?;

  let decoded = Base64UrlSafeNoPadding::decode_to_vec(jwt_header_b64, None)?;
  serde_json::from_slice(&decoded).map_err(|err| anyhow!("JWT header parse failed: {}", err))
}
