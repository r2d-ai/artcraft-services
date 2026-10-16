use std::collections::HashMap;

use anyhow::anyhow;
use errors::AnyhowResult;
use jwt_simple::algorithms::RS256PublicKey;
use jwt_simple::prelude::Base64UrlSafeNoPadding;
use jwt_simple::reexports::ct_codecs::Decoder;
use serde::Deserialize;

pub type KeyMap = HashMap<String, RS256PublicKey>;

#[derive(Deserialize)]
struct JsonKeys {
  keys: Vec<JsonKey>,
}

#[derive(Deserialize)]
struct JsonKey {
  kid: String,
  n: String,
  e: String,
  #[serde(rename = "use")]
  use_: Option<String>,
  alg: Option<String>,
}

pub fn jwk_set_to_key_map(jwks_json: &str) -> AnyhowResult<KeyMap> {
  let json_keys: JsonKeys = serde_json::from_str(jwks_json)?;
  let mut key_map = KeyMap::new();

  for key in json_keys.keys {
    if key.use_.as_deref() == Some("enc") {
      continue;
    }
    if key.alg.as_deref() != Some("RS256") {
      continue;
    }

    let n_decoded = base64_decode_url(&key.n)?;
    let e_decoded = base64_decode_url(&key.e)?;
    let public_key = RS256PublicKey::from_components(&n_decoded, &e_decoded)?
      .with_key_id(&key.kid);
    key_map.insert(key.kid, public_key);
  }

  if key_map.is_empty() {
    return Err(anyhow!("no RS256 keys in JWKS").into());
  }

  Ok(key_map)
}

fn base64_decode_url(value: &str) -> AnyhowResult<Vec<u8>> {
  Ok(Base64UrlSafeNoPadding::decode_to_vec(value, None)?)
}
