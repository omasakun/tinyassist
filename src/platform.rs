#![allow(dead_code, clippy::enum_variant_names, clippy::collapsible_if)]

use std::env;
use std::path::Path;
use std::str::FromStr;

use strum_macros::{Display, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum Os {
  Linux,
  MacOS,
  Windows,
  Unknown,
}

impl Os {
  pub fn detect() -> Self {
    Os::from_str(env::consts::OS).unwrap_or(Os::Unknown)
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum Shell {
  Bash,
  Zsh,
  Fish,
  PowerShell,
  Cmd,
  Unknown(String),
}

impl Shell {
  pub fn detect() -> Self {
    if let Ok(shell_var) = env::var("TA_SHELL") {
      return Self::from_name(&shell_var);
    }

    if let Ok(shell_path) = env::var("SHELL") {
      if let Some(shell_name) = Path::new(&shell_path).file_name().and_then(|n| n.to_str()) {
        return Self::from_name(shell_name);
      }
    }

    if Os::detect() == Os::Windows {
      if let Ok(comspec) = env::var("COMSPEC") {
        if let Some(shell_name) = Path::new(&comspec).file_name().and_then(|n| n.to_str()) {
          return Self::from_name(shell_name);
        }
      }
      return Shell::PowerShell;
    }

    Shell::Bash
  }

  pub fn from_name(name: &str) -> Self {
    let name_lower = name.to_lowercase();

    if name_lower.contains("bash") {
      Shell::Bash
    } else if name_lower.contains("zsh") {
      Shell::Zsh
    } else if name_lower.contains("fish") {
      Shell::Fish
    } else if name_lower.contains("powershell") || name_lower.contains("pwsh") {
      Shell::PowerShell
    } else if name_lower.contains("cmd") || name_lower == "cmd.exe" {
      Shell::Cmd
    } else {
      Shell::Unknown(name.to_string())
    }
  }

  pub fn get_shell_command(&self, command: &str) -> (String, Vec<String>) {
    match self {
      Shell::PowerShell => {
        ("powershell".to_string(), vec!["-Command".to_string(), command.to_string()])
      }
      Shell::Cmd => ("cmd".to_string(), vec!["/c".to_string(), command.to_string()]),
      _ => ("sh".to_string(), vec!["-c".to_string(), command.to_string()]),
    }
  }
}
