use serde_json::{Value, json};

use crate::genai::adapter::Adapter;
use crate::genai::chat::{ChatRequest, ChatResponse, ChatRole};
use crate::genai::{Error, Result};

struct AdapterConfig {
  url: fn(&str) -> String,
  headers: fn(&str) -> Vec<(&'static str, String)>,
  payload: fn(&ChatRequest, &str) -> Result<String>,
  extract: fn(&Value) -> Result<String>,
  envvar: &'static str,
}

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

    let config = match adapter_kind {
      Adapter::OpenAI => Self::openai_config(),
      Adapter::Anthropic => Self::anthropic_config(),
      Adapter::Gemini => Self::gemini_config(),
      Adapter::Groq => Self::groq_config(),
      Adapter::DeepSeek => Self::deepseek_config(),
      Adapter::Others => return Err(Error::InvalidModel(model.to_string())),
    };

    self.call_provider(&config, model, chat_req)
  }

  fn call_provider(
    &self,
    config: &AdapterConfig,
    model: &str,
    chat_req: ChatRequest,
  ) -> Result<ChatResponse> {
    let api_key =
      std::env::var(config.envvar).map_err(|_| Error::NoApiKey(config.envvar.to_string()))?;

    let url = (config.url)(model);
    let headers = (config.headers)(&api_key);
    let payload = (config.payload)(&chat_req, model)?;

    let mut request = minreq::post(&url);
    for (key, value) in headers {
      request = request.with_header(key, value);
    }

    let response =
      request.with_body(payload).send().map_err(|e| Error::RequestFailed(e.to_string()))?;

    let body = response.as_str().map_err(|e| Error::RequestFailed(e.to_string()))?;
    let json: Value = serde_json::from_str(body).map_err(|e| Error::Parse(e.to_string()))?;

    let content = (config.extract)(&json)?;

    Ok(ChatResponse { content })
  }

  fn openai_config() -> AdapterConfig {
    AdapterConfig {
      url: |_| "https://api.openai.com/v1/chat/completions".to_string(),
      headers: |api_key| {
        vec![
          ("Authorization", format!("Bearer {}", api_key)),
          ("Content-Type", "application/json".to_string()),
        ]
      },
      payload: |chat_req, model| {
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

        Ok(payload.to_string())
      },
      extract: |json| {
        json["choices"][0]["message"]["content"]
          .as_str()
          .ok_or_else(|| Error::Parse("Invalid OpenAI response structure".to_string()))
          .map(|s| s.to_string())
      },
      envvar: "OPENAI_API_KEY",
    }
  }

  fn anthropic_config() -> AdapterConfig {
    AdapterConfig {
      url: |_| "https://api.anthropic.com/v1/messages".to_string(),
      headers: |api_key| {
        vec![
          ("x-api-key", api_key.to_string()),
          ("anthropic-version", "2023-06-01".to_string()),
          ("Content-Type", "application/json".to_string()),
        ]
      },
      payload: |chat_req, model| {
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

        Ok(payload.to_string())
      },
      extract: |json| {
        json["content"][0]["text"]
          .as_str()
          .ok_or_else(|| Error::Parse("Invalid Anthropic response structure".to_string()))
          .map(|s| s.to_string())
      },
      envvar: "ANTHROPIC_API_KEY",
    }
  }

  fn gemini_config() -> AdapterConfig {
    AdapterConfig {
      url: |model| {
        format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent", model)
      },
      headers: |_| vec![("Content-Type", "application/json".to_string())],
      payload: |chat_req, _| {
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

        let payload = json!({ "contents": contents });

        Ok(payload.to_string())
      },
      extract: |json| {
        json["candidates"][0]["content"]["parts"][0]["text"]
          .as_str()
          .ok_or_else(|| Error::Parse("Invalid Gemini response structure".to_string()))
          .map(|s| s.to_string())
      },
      envvar: "GEMINI_API_KEY",
    }
  }

  fn groq_config() -> AdapterConfig {
    AdapterConfig {
      url: |_| "https://api.groq.com/openai/v1/chat/completions".to_string(),
      headers: |api_key| {
        vec![
          ("Authorization", format!("Bearer {}", api_key)),
          ("Content-Type", "application/json".to_string()),
        ]
      },
      payload: |chat_req, model| {
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

        Ok(payload.to_string())
      },
      extract: |json| {
        json["choices"][0]["message"]["content"]
          .as_str()
          .ok_or_else(|| Error::Parse("Invalid Groq response structure".to_string()))
          .map(|s| s.to_string())
      },
      envvar: "GROQ_API_KEY",
    }
  }

  fn deepseek_config() -> AdapterConfig {
    AdapterConfig {
      url: |_| "https://api.deepseek.com/chat/completions".to_string(),
      headers: |api_key| {
        vec![
          ("Authorization", format!("Bearer {}", api_key)),
          ("Content-Type", "application/json".to_string()),
        ]
      },
      payload: |chat_req, model| {
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

        Ok(payload.to_string())
      },
      extract: |json| {
        json["choices"][0]["message"]["content"]
          .as_str()
          .ok_or_else(|| Error::Parse("Invalid DeepSeek response structure".to_string()))
          .map(|s| s.to_string())
      },
      envvar: "DEEPSEEK_API_KEY",
    }
  }
}
