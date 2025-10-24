use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
  #[error("No API key available for model: {0}")]
  NoApiKey(String),
  #[error("Invalid model: {0}")]
  InvalidModel(String),
  #[error("Request failed: {0}")]
  RequestFailed(String),
  #[error("Parse error: {0}")]
  Parse(String),
}
