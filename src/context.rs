use std::env;

use crate::error::Result;
use crate::platform::{Os, Shell};

#[derive(Clone)]
pub struct ShellContext {
  pub last_command: Option<String>,
  pub exit_code: Option<i32>,
  pub working_directory: String,
  pub shell: Shell,
  pub os: Os,
}

impl ShellContext {
  pub fn current() -> Result<Self> {
    let working_directory = env::current_dir()?.to_string_lossy().to_string();
    let os = Os::detect();
    let shell = Shell::detect();

    Ok(ShellContext {
      last_command: Self::last_command(),
      exit_code: Self::exit_code(),
      working_directory,
      shell,
      os,
    })
  }

  pub fn format_for_llm(&self, fix_mode: bool) -> String {
    let exit_code_info =
      self.exit_code.map(|code| code.to_string()).unwrap_or_else(|| "n/a".to_string());

    if fix_mode {
      format!(
        "OS: {}\nShell: {}\nWorking directory: {}\nLast command: {} (exit code: {})",
        self.os,
        self.shell,
        self.working_directory,
        self.last_command.as_ref().unwrap_or(&"unknown".into()),
        exit_code_info
      )
    } else {
      format!(
        "OS: {}\nShell: {}\nWorking directory: {}",
        self.os, self.shell, self.working_directory
      )
    }
  }

  fn last_command() -> Option<String> {
    env::var("TA_LAST_COMMAND").ok().map(|cmd| cmd.trim().to_string()).filter(|cmd| !cmd.is_empty())
  }

  fn exit_code() -> Option<i32> {
    env::var("TA_EXIT_CODE").ok().and_then(|code| code.parse().ok())
  }
}
