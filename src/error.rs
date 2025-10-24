use std::env::VarError;
use std::io;

use thiserror::Error;

use crate::genai;

pub type Result<T> = core::result::Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
  #[error("{0}")]
  Context(String),
  #[error("IO error: {0}")]
  Io(#[from] io::Error),
  #[error("Environment variable error: {0}")]
  VarError(#[from] VarError),
  #[error("Clipboard error: {0}")]
  Clipboard(#[from] arboard::Error),
  #[error("API error: {0}")]
  Api(#[from] genai::Error),
}
