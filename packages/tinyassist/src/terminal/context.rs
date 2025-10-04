use anyhow::{Context, Result};
use std::env;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct TerminalContext {
    pub last_command: String,
    pub exit_code: Option<i32>,
    pub working_directory: String,
    pub shell: String,
}

impl TerminalContext {
    pub fn format_for_llm(&self) -> String {
        let exit_code_info = match self.exit_code {
            Some(code) => code.to_string(),
            None => "N/A".to_string(),
        };

        format!(
            "Last command: {}\nExit code: {}\n\nWorking directory: {}\nShell: {}",
            self.last_command, exit_code_info, self.working_directory, self.shell
        )
    }
}

pub fn get_terminal_context() -> Result<TerminalContext> {
    let working_directory = env::current_dir()
        .context("Failed to get current directory")?
        .to_string_lossy()
        .to_string();

    let shell = get_shell();
    let last_command = get_last_command().unwrap_or_default();
    let exit_code = get_exit_code();

    Ok(TerminalContext {
        last_command,
        exit_code,
        working_directory,
        shell,
    })
}

fn get_shell() -> String {
    // First check TA_SHELL (set by shell integration)
    if let Ok(shell) = env::var("TA_SHELL") {
        return shell;
    }

    // Fall back to SHELL environment variable
    env::var("SHELL")
        .ok()
        .and_then(|path| {
            Path::new(&path)
                .file_name()
                .and_then(|name| name.to_str())
                .map(String::from)
        })
        .unwrap_or_else(|| "bash".to_string())
}

fn get_last_command() -> Option<String> {
    env::var("TA_LAST_COMMAND")
        .ok()
        .map(|cmd| cmd.trim().to_string())
        .filter(|cmd| !cmd.is_empty())
}

fn get_exit_code() -> Option<i32> {
    env::var("TA_EXIT_CODE").ok().and_then(|code| code.parse().ok())
}
