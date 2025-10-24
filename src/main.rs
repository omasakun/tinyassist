#![allow(clippy::type_complexity)]

mod context;
mod error;
mod genai;
mod platform;
mod terminal;

use std::path::PathBuf;
use std::process::ExitCode;

use argh::FromArgs;
use colored::Colorize;
use directories::ProjectDirs;
use indoc::indoc;
use terminal::{ChatAction, UserAction, copy_to_clipboard, execute_command, prompt};

use crate::context::ShellContext;
use crate::error::{AppError, Result};
use crate::genai::{ChatMessage, ChatRequest, Client};
use crate::platform::Shell;

const DEFAULT_MODEL: &str = "gpt-4o-mini";

const GENERATE_SYSTEM_PROMPT: &str = indoc! {"
  You are an AI designed to help users identify the precise shell command they need.
  Always provide a concise shell command.

  Important:
  - Think hard about the command you are going to provide, and make sure it is the best one.
  - No shebang, no explanations, no extra text. Just the command.
"};

const FIX_SYSTEM_PROMPT: &str = indoc! {"
  You are a helpful AI that fixes failed shell commands.
  Analyze the provided command and suggest a corrected version.

  Important:
  - Only provide the corrected command, no explanations, no shebang, no extra text.
  - Fix common issues like: missing sudo, typos, wrong flags, missing files/directories
  - Keep the original intent of the command
  - If the command seems correct, suggest checking prerequisites or dependencies
"};

const EXPLAIN_SYSTEM_PROMPT: &str =
  "You are a helpful AI that explains shell commands very concisely. Keep it under 50 words.";

/// Generate shell commands from natural language descriptions
#[derive(FromArgs)]
#[argh(help_triggers("--help"))]
pub struct Args {
  /// model to use
  #[argh(option, default = "DEFAULT_MODEL.to_string()")]
  pub model: String,

  /// fix the last command
  #[argh(switch, short = 'f')]
  pub fix: bool,

  /// free form conversation
  #[argh(switch)]
  pub chat: bool,

  /// show config info
  #[argh(switch)]
  pub info: bool,

  /// generate shell integration script
  #[argh(option)]
  pub init: Option<String>,

  /// alias name for the shell function (used with --init)
  #[argh(option)]
  pub alias: Option<String>,

  /// natural language description of the command you want
  #[argh(positional, greedy)]
  pub prompt: Vec<String>,
}

impl Args {
  fn prompt(&self) -> String {
    self.prompt.join(" ")
  }

  fn config_dir() -> Option<PathBuf> {
    ProjectDirs::from("", "", "tinyassist").map(|proj_dirs| proj_dirs.config_dir().to_path_buf())
  }
}

fn main() -> ExitCode {
  match main_inner() {
    Ok(code) => code,
    Err(e) => {
      eprintln!("{} {}", "Error:".red(), e);
      ExitCode::from(1)
    }
  }
}

fn main_inner() -> Result<ExitCode> {
  load_env();

  let args: Args = argh::from_env();

  // Handle --init option
  if let Some(shell_name) = &args.init {
    let alias_name = args.alias.as_deref().unwrap_or("ta");
    print_shell_integration(shell_name, alias_name)?;
    return Ok(ExitCode::SUCCESS);
  }

  // Handle --info flag
  if args.info {
    if let Some(config_dir) = config_dir() {
      println!("Config directory: {}", config_dir.display());
      let env_file = config_dir.join(".env");
      if env_file.exists() {
        println!("Config .env file: {} {}", env_file.display(), "(exists)".green());
      } else {
        println!("Config .env file: {} {}", env_file.display(), "(not found)".yellow());
      }
    } else {
      println!("{}", "Unable to determine configuration directory".red());
    }
    println!("\nContext for LLM: \n{}", ShellContext::current()?.format_for_llm(true));
    return Ok(ExitCode::SUCCESS);
  }

  let exit_code = if args.chat {
    let initial_prompt = if args.prompt.is_empty() { None } else { Some(args.prompt()) };
    handle_chat_mode(&args.model, initial_prompt)?
  } else if args.fix {
    handle_fix_mode(&args.model, &args.prompt())?
  } else {
    let prompt_text = if args.prompt.is_empty() { prompt("Prompt: ")? } else { args.prompt() };
    handle_normal_mode(&args.model, &prompt_text)?
  };

  Ok(ExitCode::from(exit_code as u8))
}

/// Handle chat mode: free form conversation with LLM
fn handle_chat_mode(model: &str, initial_prompt: Option<String>) -> Result<i32> {
  let client = Client::new();
  let mut messages: Vec<ChatMessage> = Vec::new();

  const CHAT_SYSTEM_PROMPT: &str =
    "You are a helpful AI assistant. Provide concise and accurate responses.";

  messages.push(ChatMessage::system(CHAT_SYSTEM_PROMPT));

  let mut initial_input = initial_prompt;

  loop {
    let user_input = if let Some(input) = initial_input.take() { input } else { prompt("You: ")? };

    if user_input.trim().is_empty() {
      continue;
    }

    messages.push(ChatMessage::user(&user_input));

    let chat_req = ChatRequest::new(messages.clone());
    let chat_res = client.exec_chat(model, chat_req)?;

    let response_text = chat_res
      .first_text()
      .ok_or_else(|| AppError::Context("No response text from AI".to_string()))?
      .to_string();

    println!("{} {}", "AI:".green(), response_text);

    messages.push(ChatMessage::assistant(&response_text));

    loop {
      let action = ChatAction::ask()?;

      match action {
        ChatAction::Copy => {
          copy_to_clipboard(&response_text)?;
          println!("{}", "Copied to clipboard".green());
          break;
        }
        ChatAction::Revert => {
          messages.pop();
          messages.pop();
          if messages.len() > 1 {
            println!("{}", "Reverted. You can enter a different prompt".yellow());
            let corrected_input = prompt("You: ")?;

            if !corrected_input.trim().is_empty() {
              messages.push(ChatMessage::user(&corrected_input));

              let chat_req = ChatRequest::new(messages.clone());
              let chat_res = client.exec_chat(model, chat_req)?;

              let new_response_text = chat_res
                .first_text()
                .ok_or_else(|| AppError::Context("No response text from AI".to_string()))?
                .to_string();

              println!("{} {}", "AI:".green(), new_response_text);

              messages.pop();
              messages.push(ChatMessage::assistant(&new_response_text));
            }
          }
          continue;
        }
        ChatAction::Next => break,
        ChatAction::Abort => return Ok(0),
      }
    }
  }
}

/// Handle --fix mode: fix the last command
fn handle_fix_mode(model: &str, prompt: &str) -> Result<i32> {
  let context = ShellContext::current()?;

  println!(
    "{}",
    format!("Last command: {}", context.last_command.as_ref().unwrap_or(&"unknown".into()))
      .yellow()
  );

  let client = Client::new();

  let contextual_prompt = if prompt.is_empty() {
    context.format_for_llm(true)
  } else {
    format!("{}\n\nFix instructions: {}", context.format_for_llm(true), prompt)
  };

  let messages =
    vec![ChatMessage::system(FIX_SYSTEM_PROMPT), ChatMessage::user(&contextual_prompt)];

  let chat_req = ChatRequest::new(messages.clone());
  let chat_res = client.exec_chat(model, chat_req)?;

  let response_text = chat_res
    .first_text()
    .ok_or_else(|| AppError::Context("No response text from AI".to_string()))?;

  let command = response_text.trim().to_string();
  let mut messages_with_response = messages;
  messages_with_response.push(ChatMessage::assistant(&command));
  command_interaction_loop(command, messages_with_response, model)
}

/// Handle normal mode: generate command from prompt
fn handle_normal_mode(model: &str, prompt: &str) -> Result<i32> {
  if prompt.trim().is_empty() {
    return Err(AppError::Context("Please provide a prompt".to_string()));
  }

  let context = ShellContext::current()?;

  let (command, messages) = generate_command(model, prompt, &context)?;
  command_interaction_loop(command, messages, model)
}

fn generate_command(
  model: &str,
  user_prompt: &str,
  context: &context::ShellContext,
) -> Result<(String, Vec<ChatMessage>)> {
  let client = Client::new();

  let contextual_prompt = format!("{}\n\n{}", user_prompt, context.format_for_llm(false));
  let messages =
    vec![ChatMessage::system(GENERATE_SYSTEM_PROMPT), ChatMessage::user(&contextual_prompt)];
  let chat_req = ChatRequest::new(messages.clone());

  let chat_res = client.exec_chat(model, chat_req)?;

  let response_text = chat_res
    .first_text()
    .ok_or_else(|| AppError::Context("No response text from AI".to_string()))?;

  let command = response_text.trim().to_string();
  let mut messages_with_response = messages;
  messages_with_response.push(ChatMessage::assistant(&command));

  Ok((command, messages_with_response))
}

fn explain_command(model: &str, command: &str) -> Result<String> {
  let client = Client::new();

  let explain_prompt =
    format!("Explain this shell command in 1-2 simple sentences:\n\n{}", command);

  let chat_req = ChatRequest::new(vec![
    ChatMessage::system(EXPLAIN_SYSTEM_PROMPT),
    ChatMessage::user(&explain_prompt),
  ]);

  let chat_res = client.exec_chat(model, chat_req)?;

  let response_text = chat_res
    .first_text()
    .ok_or_else(|| AppError::Context("No explanation text from AI".to_string()))?;

  Ok(response_text.trim().to_string())
}

fn refine_command(
  model: &str,
  mut messages: Vec<ChatMessage>,
  refinement: &str,
) -> Result<(String, Vec<ChatMessage>)> {
  let client = Client::new();

  messages.push(ChatMessage::user(refinement));

  let chat_req = ChatRequest::new(messages.clone());

  let chat_res = client.exec_chat(model, chat_req)?;

  let response_text = chat_res
    .first_text()
    .ok_or_else(|| AppError::Context("No response text from AI".to_string()))?;

  let command = response_text.trim().to_string();
  messages.push(ChatMessage::assistant(&command));

  Ok((command, messages))
}

/// Load .env file if it exists
fn load_env() {
  let env_file = Args::config_dir().map(|path| path.join(".env"));

  if let Some(env_file) = env_file {
    let _ = dotenvy::from_path(&env_file);
  } else {
    let _ = dotenvy::dotenv();
  }
}

/// Get the configuration directory for tinyassist
fn config_dir() -> Option<PathBuf> {
  Args::config_dir()
}

fn print_response(command: &str) {
  println!("{} {}", ">".blue(), command);
}

fn print_bash_integration(alias_name: &str) {
  println!(
    "{}",
    indoc! {"
      # tinyassist shell integration for bash
      # Add this to your ~/.bashrc or ~/.bash_profile:
      # eval \"$(tinyassist --init bash)\"

      function {alias}() {
          export TA_EXIT_CODE=$?
          export TA_SHELL=bash
          export TA_LAST_COMMAND=$(fc -ln -1 2>/dev/null)

          tinyassist \"$@\"

          unset TA_SHELL
          unset TA_EXIT_CODE
          unset TA_LAST_COMMAND
      }
    "}
    .replace("{alias}", alias_name)
  );
}

fn print_zsh_integration(alias_name: &str) {
  println!(
    "{}",
    indoc! {"
      # tinyassist shell integration for zsh
      # Add this to your ~/.zshrc:
      # eval \"$(tinyassist --init zsh)\"

      function {alias}() {
          export TA_EXIT_CODE=$?
          export TA_SHELL=zsh
          export TA_LAST_COMMAND=$(fc -ln -1 2>/dev/null)

          tinyassist \"$@\"

          unset TA_SHELL
          unset TA_EXIT_CODE
          unset TA_LAST_COMMAND
      }
    "}
    .replace("{alias}", alias_name)
  );
}

fn print_fish_integration(alias_name: &str) {
  println!(
    "{}",
    indoc! {"
      # tinyassist shell integration for fish
      # Add this to your ~/.config/fish/config.fish:
      # tinyassist --init fish | source

      function {alias}
          set -x TA_EXIT_CODE $status
          set -x TA_SHELL fish
          set -x TA_LAST_COMMAND (builtin history -1)

          tinyassist $argv

          set -e TA_SHELL
          set -e TA_EXIT_CODE
          set -e TA_LAST_COMMAND
      end
    "}
    .replace("{alias}", alias_name)
  );
}

fn print_powershell_integration(alias_name: &str) {
  println!(
    "{}",
    indoc! {"
      # tinyassist shell integration for PowerShell
      # Add this to your $PROFILE:
      # Invoke-Expression (& tinyassist --init pwsh | Out-String)

      function {alias} {
          $env:TA_EXIT_CODE = if ($?) { 0 } else { $LASTEXITCODE }
          $env:TA_SHELL = \"pwsh\"
          $env:TA_LAST_COMMAND = @(Get-History -Count 1).CommandLine

          & tinyassist @args

          Remove-Item Env:TA_SHELL -ErrorAction SilentlyContinue
          Remove-Item Env:TA_EXIT_CODE -ErrorAction SilentlyContinue
          Remove-Item Env:TA_LAST_COMMAND -ErrorAction SilentlyContinue
      }
    "}
    .replace("{alias}", alias_name)
  );
}

/// Print shell integration script for the specified shell
fn print_shell_integration(shell_name: &str, alias_name: &str) -> Result<()> {
  let shell = Shell::from_name(shell_name);

  match shell {
    Shell::Bash => {
      print_bash_integration(alias_name);
      Ok(())
    }
    Shell::Zsh => {
      print_zsh_integration(alias_name);
      Ok(())
    }
    Shell::Fish => {
      print_fish_integration(alias_name);
      Ok(())
    }
    Shell::PowerShell => {
      print_powershell_integration(alias_name);
      Ok(())
    }
    Shell::Cmd => {
      Err(AppError::Context("Shell integration not yet supported for cmd.exe".to_string()))
    }
    Shell::Unknown(name) => Err(AppError::Context(format!(
      "Unsupported shell: {}. Supported shells: bash, zsh, fish, powershell",
      name
    ))),
  }
}

fn run_user_action(
  action: UserAction,
  command: &str,
  messages: &[ChatMessage],
  model: &str,
) -> Result<(i32, Option<(String, Vec<ChatMessage>)>)> {
  match action {
    UserAction::Copy => {
      copy_to_clipboard(command)?;
      println!("{}", "Copied to clipboard".green());
      Ok((0, None))
    }
    UserAction::Run => {
      let exit_code = execute_command(command)?;
      Ok((exit_code, None))
    }
    UserAction::Explain => {
      let explanation = explain_command(model, command)?;
      println!("\n{}", "Explanation:".green());
      println!("{}\n", explanation);
      Ok((0, None))
    }
    UserAction::Refine => {
      let refinement = prompt("Instructions: ")?;
      if refinement.trim().is_empty() {
        println!("{}", "No refinement provided.".yellow());
        Ok((0, None))
      } else {
        let refined_result = refine_command(model, messages.to_vec(), &refinement)?;
        Ok((0, Some(refined_result)))
      }
    }
    UserAction::Abort => Ok((1, None)),
  }
}

/// Interactive command loop that handles user actions
fn command_interaction_loop(
  initial_command: String,
  initial_messages: Vec<ChatMessage>,
  model: &str,
) -> Result<i32> {
  let mut current_command = initial_command;
  let mut current_messages = initial_messages;

  loop {
    print_response(&current_command);

    let action = UserAction::ask()?;
    let (exit_code, new_result) =
      run_user_action(action, &current_command, &current_messages, model)?;

    match action {
      UserAction::Copy | UserAction::Run => {
        return Ok(exit_code);
      }
      UserAction::Explain => {
        continue;
      }
      UserAction::Refine => {
        if let Some((refined_command, refined_messages)) = new_result {
          current_command = refined_command;
          current_messages = refined_messages;
        }
        continue;
      }
      UserAction::Abort => {
        return Ok(1);
      }
    }
  }
}
