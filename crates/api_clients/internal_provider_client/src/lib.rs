use anyhow::anyhow;
use serde::Deserialize;
use url::Url;

#[derive(Clone, Debug)]
pub struct InternalProviderClient {
  base_url: String,
  api_key: String,
  default_model: String,
}

impl InternalProviderClient {
  pub fn new(base_url: String, api_key: String, default_model: String) -> Self {
    Self {
      base_url: base_url.trim_end_matches('/').to_string(),
      api_key,
      default_model,
    }
  }

  pub async fn generate_image_png_bytes(
    &self,
    prompt: &str,
    maybe_model: Option<&str>,
  ) -> anyhow::Result<Vec<u8>> {
    let model = maybe_model.unwrap_or(&self.default_model);
    let endpoint = format!("{}/v1/images/generations", self.base_url);

    let response = reqwest::Client::new()
      .post(endpoint)
      .bearer_auth(&self.api_key)
      .json(&serde_json::json!({
        "model": model,
        "prompt": prompt,
        "response_format": "b64_json",
        "size": "1024x1024",
      }))
      .send()
      .await?;

    if !response.status().is_success() {
      let body = response.text().await.unwrap_or_default();
      return Err(anyhow!("internal provider image error: {}", body));
    }

    let payload = response.json::<ImageGenerationResponse>().await?;
    let entry = payload
      .data
      .first()
      .ok_or_else(|| anyhow!("internal provider returned no image data"))?;

    // Providers commonly return either inline base64 or a hosted URL;
    // accept both so any OpenAI-compatible backend works.
    let bytes = if let Some(b64) = &entry.b64_json {
      base64::decode(b64)?
    } else if let Some(url) = &entry.url {
      let downloaded = reqwest::Client::new()
        .get(url)
        .send()
        .await?
        .error_for_status()?;
      downloaded.bytes().await?.to_vec()
    } else {
      return Err(anyhow!("internal provider returned no image data"));
    };

    Ok(bytes)
  }

  /// OpenAI-compatible chat completion (for LLM routing).
  pub async fn chat_completion(
    &self,
    model: &str,
    user_prompt: &str,
  ) -> anyhow::Result<String> {
    let endpoint = format!("{}/v1/chat/completions", self.base_url);
    let response = reqwest::Client::new()
      .post(endpoint)
      .bearer_auth(&self.api_key)
      .json(&serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": user_prompt}],
      }))
      .send()
      .await?;

    if !response.status().is_success() {
      let body = response.text().await.unwrap_or_default();
      return Err(anyhow!("internal provider chat error: {}", body));
    }

    let payload = response.json::<ChatCompletionResponse>().await?;
    let content = payload
      .choices
      .first()
      .and_then(|choice| choice.message.content.clone())
      .ok_or_else(|| anyhow!("internal provider returned no chat content"))?;
    Ok(content)
  }

  pub fn validate_base_url(base_url: &str) -> anyhow::Result<()> {
    let parsed = Url::parse(base_url)?;
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
      return Err(anyhow!("internal provider URL must be http(s)"));
    }
    Ok(())
  }
}

#[derive(Deserialize)]
struct ImageGenerationResponse {
  data: Vec<ImageDataEntry>,
}

#[derive(Deserialize)]
struct ImageDataEntry {
  b64_json: Option<String>,
  url: Option<String>,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
  choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
  message: ChatMessage,
}

#[derive(Deserialize)]
struct ChatMessage {
  content: Option<String>,
}
