use serde_json::json;

use crate::genai::adapter::Adapter;
use crate::genai::chat::{ChatRequest, ChatResponse, ChatRole};
use crate::genai::{Error, Result};

/// High-level LLM client supporting multiple providers.
///
/// Automatically detects the appropriate adapter based on the model name
pub struct Client;

impl Client {
  pub fn new() -> Self {
    Self
  }

  pub fn exec_chat(&self, model: &str, chat_req: ChatRequest) -> Result<ChatResponse> {
    let adapter_kind = Adapter::from_model_name(model);

    match adapter_kind {
      Adapter::OpenAI => self.call_openai(model, chat_req),
      Adapter::Anthropic => self.call_anthropic(model, chat_req),
      Adapter::Gemini => self.call_gemini(model, chat_req),
      Adapter::Groq => self.call_groq(model, chat_req),
      Adapter::DeepSeek => self.call_deepseek(model, chat_req),
      Adapter::Others => Err(Error::InvalidModel(model.to_string())),
    }
  }

  fn call_openai(&self, model: &str, chat_req: ChatRequest) -> Result<ChatResponse> {
    let api_key =
      std::env::var("OPENAI_API_KEY").map_err(|_| Error::NoApiKey("OpenAI".to_string()))?;

    let messages = chat_req
      .messages
      .iter()
      .map(|msg| {
        json!({
          "role": msg.role.to_string(),
          "content": msg.content,
        })
      })
      .collect::<Vec<_>>();

    let payload = json!({
      "model": model,
      "messages": messages,
    });

    let response = minreq::post("https://api.openai.com/v1/chat/completions")
      .with_header("Authorization", format!("Bearer {}", api_key))
      .with_header("Content-Type", "application/json")
      .with_body(payload.to_string())
      .send()
      .map_err(|e| Error::RequestFailed(e.to_string()))?;

    let body = response.as_str().map_err(|e| Error::RequestFailed(e.to_string()))?;
    let json: serde_json::Value =
      serde_json::from_str(body).map_err(|e| Error::Parse(e.to_string()))?;

    let content = json["choices"][0]["message"]["content"]
      .as_str()
      .ok_or_else(|| Error::Parse("Invalid OpenAI response structure".to_string()))?
      .to_string();

    Ok(ChatResponse { content })
  }

  fn call_anthropic(&self, model: &str, chat_req: ChatRequest) -> Result<ChatResponse> {
    let api_key =
      std::env::var("ANTHROPIC_API_KEY").map_err(|_| Error::NoApiKey("Anthropic".to_string()))?;

    let messages = chat_req
      .messages
      .iter()
      .filter(|msg| msg.role != ChatRole::System)
      .map(|msg| {
        json!({
          "role": msg.role.to_string(),
          "content": msg.content,
        })
      })
      .collect::<Vec<_>>();

    let system = chat_req
      .messages
      .iter()
      .find(|msg| msg.role == ChatRole::System)
      .map(|msg| msg.content.clone());

    let mut payload = json!({
      "model": model,
      "max_tokens": 1024,
      "messages": messages,
    });

    if let Some(system) = system {
      payload["system"] = json!(system);
    }

    let response = minreq::post("https://api.anthropic.com/v1/messages")
      .with_header("x-api-key", &api_key)
      .with_header("anthropic-version", "2023-06-01")
      .with_header("Content-Type", "application/json")
      .with_body(payload.to_string())
      .send()
      .map_err(|e| Error::RequestFailed(e.to_string()))?;

    let body = response.as_str().map_err(|e| Error::RequestFailed(e.to_string()))?;
    let json: serde_json::Value =
      serde_json::from_str(body).map_err(|e| Error::Parse(e.to_string()))?;

    let content = json["content"][0]["text"]
      .as_str()
      .ok_or_else(|| Error::Parse("Invalid Anthropic response structure".to_string()))?
      .to_string();

    Ok(ChatResponse { content })
  }

  fn call_gemini(&self, model: &str, chat_req: ChatRequest) -> Result<ChatResponse> {
    let api_key =
      std::env::var("GEMINI_API_KEY").map_err(|_| Error::NoApiKey("Gemini".to_string()))?;

    let contents = chat_req
      .messages
      .iter()
      .map(|msg| {
        json!({
          "role": if msg.role == ChatRole::User { "user" } else { "model" },
          "parts": [{"text": msg.content}],
        })
      })
      .collect::<Vec<_>>();

    let payload = json!({
      "contents": contents,
    });

    let url = format!(
      "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
      model, api_key
    );

    let response = minreq::post(&url)
      .with_header("Content-Type", "application/json")
      .with_body(payload.to_string())
      .send()
      .map_err(|e| Error::RequestFailed(e.to_string()))?;

    let body = response.as_str().map_err(|e| Error::RequestFailed(e.to_string()))?;
    let json: serde_json::Value =
      serde_json::from_str(body).map_err(|e| Error::Parse(e.to_string()))?;

    let content = json["candidates"][0]["content"]["parts"][0]["text"]
      .as_str()
      .ok_or_else(|| Error::Parse("Invalid Gemini response structure".to_string()))?
      .to_string();

    Ok(ChatResponse { content })
  }

  fn call_groq(&self, model: &str, chat_req: ChatRequest) -> Result<ChatResponse> {
    let api_key = std::env::var("GROQ_API_KEY").map_err(|_| Error::NoApiKey("Groq".to_string()))?;

    let messages = chat_req
      .messages
      .iter()
      .map(|msg| {
        json!({
          "role": msg.role.to_string(),
          "content": msg.content,
        })
      })
      .collect::<Vec<_>>();

    let payload = json!({
      "model": model,
      "messages": messages,
    });

    let response = minreq::post("https://api.groq.com/openai/v1/chat/completions")
      .with_header("Authorization", format!("Bearer {}", api_key))
      .with_header("Content-Type", "application/json")
      .with_body(payload.to_string())
      .send()
      .map_err(|e| Error::RequestFailed(e.to_string()))?;

    let body = response.as_str().map_err(|e| Error::RequestFailed(e.to_string()))?;
    let json: serde_json::Value =
      serde_json::from_str(body).map_err(|e| Error::Parse(e.to_string()))?;

    let content = json["choices"][0]["message"]["content"]
      .as_str()
      .ok_or_else(|| Error::Parse("Invalid Groq response structure".to_string()))?
      .to_string();

    Ok(ChatResponse { content })
  }

  fn call_deepseek(&self, model: &str, chat_req: ChatRequest) -> Result<ChatResponse> {
    let api_key =
      std::env::var("DEEPSEEK_API_KEY").map_err(|_| Error::NoApiKey("DeepSeek".to_string()))?;

    let messages = chat_req
      .messages
      .iter()
      .map(|msg| {
        json!({
          "role": msg.role.to_string(),
          "content": msg.content,
        })
      })
      .collect::<Vec<_>>();

    let payload = json!({
      "model": model,
      "messages": messages,
    });

    let response = minreq::post("https://api.deepseek.com/chat/completions")
      .with_header("Authorization", format!("Bearer {}", api_key))
      .with_header("Content-Type", "application/json")
      .with_body(payload.to_string())
      .send()
      .map_err(|e| Error::RequestFailed(e.to_string()))?;

    let body = response.as_str().map_err(|e| Error::RequestFailed(e.to_string()))?;
    let json: serde_json::Value =
      serde_json::from_str(body).map_err(|e| Error::Parse(e.to_string()))?;

    let content = json["choices"][0]["message"]["content"]
      .as_str()
      .ok_or_else(|| Error::Parse("Invalid DeepSeek response structure".to_string()))?
      .to_string();

    Ok(ChatResponse { content })
  }
}
