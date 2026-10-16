use errors::AnyhowResult;
use jwt_simple::prelude::Base64UrlSafeNoPadding;
use jwt_simple::reexports::ct_codecs::Encoder;
use rand::{rngs::OsRng, TryRngCore};
use sha2::{Digest, Sha256};

pub struct PkcePair {
  pub code_verifier: String,
  pub code_challenge: String,
}

pub fn generate_pkce_pair() -> AnyhowResult<PkcePair> {
  let mut verifier_bytes = [0u8; 32];
  OsRng.try_fill_bytes(&mut verifier_bytes)?;
  let code_verifier = Base64UrlSafeNoPadding::encode_to_string(&verifier_bytes)
    .map_err(|err| anyhow::anyhow!("pkce encode failed: {}", err))?;
  let digest = Sha256::digest(code_verifier.as_bytes());
  let code_challenge = Base64UrlSafeNoPadding::encode_to_string(&digest)
    .map_err(|err| anyhow::anyhow!("pkce encode failed: {}", err))?;
  Ok(PkcePair {
    code_verifier,
    code_challenge,
  })
}
