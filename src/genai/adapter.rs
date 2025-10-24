use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum Adapter {
  OpenAI,
  Anthropic,
  Gemini,
  Groq,
  DeepSeek,
  Others,
}

impl Adapter {
  pub fn from_model_name(model: &str) -> Self {
    let model_lower = model.to_lowercase();

    if model_lower.contains("gpt") || model_lower.contains("o1") || model_lower.contains("o3") {
      Adapter::OpenAI
    } else if model_lower.contains("claude") {
      Adapter::Anthropic
    } else if model_lower.contains("gemini") {
      Adapter::Gemini
    } else if model_lower.contains("groq") {
      Adapter::Groq
    } else if model_lower.contains("deepseek") {
      Adapter::DeepSeek
    } else {
      Adapter::Others
    }
  }
}
