use std::env;
use std::path::Path;

use anyhow::{Context as _, Result};

#[derive(Debug, Clone)]
pub struct ShellContext {
  pub last_command: String,
  pub exit_code: Option<i32>,
  pub working_directory: String,
  pub shell: String,
}

impl ShellContext {
  pub fn current() -> Result<Self> {
    let working_directory =
      env::current_dir().context("Failed to get current directory")?.to_string_lossy().to_string();

    Ok(ShellContext {
      last_command: Self::last_command(),
      exit_code: Self::exit_code(),
      working_directory,
      shell: Self::shell(),
    })
  }

  pub fn format_for_llm(&self) -> String {
    let exit_code_info =
      self.exit_code.map(|code| code.to_string()).unwrap_or_else(|| "N/A".to_string());

    format!(
      "Last command: {}\nExit code: {}\n\nWorking directory: {}\nShell: {}",
      self.last_command, exit_code_info, self.working_directory, self.shell
    )
  }

  fn shell() -> String {
    env::var("TA_SHELL")
      .or_else(|_| env::var("SHELL"))
      .ok()
      .and_then(|path| {
        Path::new(&path).file_name().and_then(|name| name.to_str()).map(String::from)
      })
      .unwrap_or_else(|| "bash".to_string())
  }

  fn last_command() -> String {
    env::var("TA_LAST_COMMAND")
      .ok()
      .map(|cmd| cmd.trim().to_string())
      .filter(|cmd| !cmd.is_empty())
      .unwrap_or_default()
  }

  fn exit_code() -> Option<i32> {
    env::var("TA_EXIT_CODE").ok().and_then(|code| code.parse().ok())
  }
}
