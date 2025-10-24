use serde::{Deserialize, Serialize};
use strum_macros::Display;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
  pub role: ChatRole,
  pub content: String,
}

impl ChatMessage {
  pub fn system(content: impl Into<String>) -> Self {
    Self { role: ChatRole::System, content: content.into() }
  }

  pub fn user(content: impl Into<String>) -> Self {
    Self { role: ChatRole::User, content: content.into() }
  }

  pub fn assistant(content: impl Into<String>) -> Self {
    Self { role: ChatRole::Assistant, content: content.into() }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Display)]
#[strum(serialize_all = "lowercase")]
pub enum ChatRole {
  System,
  User,
  Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
  pub messages: Vec<ChatMessage>,
}

impl ChatRequest {
  pub fn new(messages: Vec<ChatMessage>) -> Self {
    Self { messages }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
  pub content: String,
}

impl ChatResponse {
  pub fn first_text(&self) -> Option<&str> {
    Some(&self.content)
  }
}
