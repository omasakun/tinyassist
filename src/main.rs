mod context;
mod terminal;

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context as _, Result};
use clap::Parser;
use colored::Colorize;
use directories::ProjectDirs;
use genai::Client;
use genai::chat::{ChatMessage, ChatRequest};
use terminal::{UserAction, copy_to_clipboard, error, execute_command, prompt};

use crate::context::ShellContext;

const DEFAULT_MODEL: &str = "gpt-4o-mini";

const GENERATE_SYSTEM_PROMPT: &str = r#"You are an AI designed to help users identify the precise shell command they need.
Always provide a concise shell command.

Important:
- Think hard about the command you are going to provide, and make sure it is the best one.
- No shebang, no explanations, no extra text. Just the command.

User uses Linux/macOS"#;

const FIX_SYSTEM_PROMPT: &str = r#"You are a helpful AI that fixes failed shell commands.
Analyze the provided command and suggest a corrected version.

Important:
- Only provide the corrected command, no explanations, no shebang, no extra text.
- Fix common issues like: missing sudo, typos, wrong flags, missing files/directories
- Keep the original intent of the command
- If the command seems correct, suggest checking prerequisites or dependencies

User uses Linux/macOS"#;

const EXPLAIN_SYSTEM_PROMPT: &str =
  "You are a helpful AI that explains shell commands very concisely. Keep it under 50 words.";

/// CLI Arguments
#[derive(Parser, Debug)]
#[command(name = "tinyassist")]
#[command(about = "Generate shell commands from natural language descriptions")]
#[command(version = "0.1.0")]
pub struct Args {
  /// Model to use
  #[arg(long, default_value = DEFAULT_MODEL)]
  pub model: String,

  /// Fix the last command
  #[arg(long)]
  pub fix: bool,

  /// Show config info
  #[arg(long)]
  pub info: bool,

  /// Generate shell integration script
  #[arg(long)]
  pub init: bool,

  /// Natural language description of the command you want
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

#[tokio::main]
async fn main() -> Result<ExitCode> {
  load_env();

  let args = Args::parse();

  // Handle --init flag
  if args.init {
    print_shell_integration(args.prompt.first().map(|s| s.as_str()).unwrap_or("ta"))?;
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
    return Ok(ExitCode::SUCCESS);
  }

  let exit_code = if args.fix {
    handle_fix_mode(&args.model).await?
  } else {
    handle_normal_mode(&args.model, &args.prompt()).await?
  };

  Ok(ExitCode::from(exit_code as u8))
}

/// Handle --fix mode: fix the last command
async fn handle_fix_mode(model: &str) -> Result<i32> {
  let context = ShellContext::current().context("Failed to get terminal context")?;

  println!("{}", format!("Last command: {}", context.last_command).yellow());

  let (command, messages) = fix_last_command(model, &context).await?;
  command_interaction_loop(command, messages, model).await
}

/// Handle normal mode: generate command from prompt
async fn handle_normal_mode(model: &str, prompt: &str) -> Result<i32> {
  if prompt.trim().is_empty() {
    error("Please provide a prompt");
    return Ok(1);
  }

  let context = ShellContext::current().context("Failed to get terminal context")?;

  let (command, messages) = generate_command(model, prompt, &context).await?;
  command_interaction_loop(command, messages, model).await
}

async fn fix_last_command(
  model: &str,
  context: &context::ShellContext,
) -> Result<(String, Vec<ChatMessage>)> {
  let client = Client::default();

  let contextual_prompt = context.format_for_llm();

  let messages =
    vec![ChatMessage::system(FIX_SYSTEM_PROMPT), ChatMessage::user(&contextual_prompt)];

  let chat_req = ChatRequest::new(messages.clone());

  let chat_res = client
    .exec_chat(model, chat_req, None)
    .await
    .context("Failed to execute fix command request")?;

  let response_text = chat_res.first_text().context("No response text from AI")?;

  Ok((response_text.trim().to_string(), messages))
}

async fn generate_command(
  model: &str,
  user_prompt: &str,
  context: &context::ShellContext,
) -> Result<(String, Vec<ChatMessage>)> {
  let client = Client::default();

  let contextual_prompt = format!(
    "{}\n\nContext:\nWorking directory: {}\nShell: {}",
    user_prompt, context.working_directory, context.shell
  );

  let messages =
    vec![ChatMessage::system(GENERATE_SYSTEM_PROMPT), ChatMessage::user(&contextual_prompt)];
  let chat_req = ChatRequest::new(messages.clone());

  let chat_res =
    client.exec_chat(model, chat_req, None).await.context("Failed to execute chat request")?;

  let response_text = chat_res.first_text().context("No response text from AI")?;

  Ok((response_text.trim().to_string(), messages))
}

async fn explain_command(model: &str, command: &str) -> Result<String> {
  let client = Client::default();

  let explain_prompt =
    format!("Explain this shell command in 1-2 simple sentences:\n\n{}", command);

  let chat_req = ChatRequest::new(vec![
    ChatMessage::system(EXPLAIN_SYSTEM_PROMPT),
    ChatMessage::user(&explain_prompt),
  ]);

  let chat_res =
    client.exec_chat(model, chat_req, None).await.context("Failed to execute explain request")?;

  let response_text = chat_res.first_text().context("No explanation text from AI")?;

  Ok(response_text.trim().to_string())
}

async fn refine_command(
  model: &str,
  mut messages: Vec<ChatMessage>,
  refinement: &str,
) -> Result<(String, Vec<ChatMessage>)> {
  let client = Client::default();

  messages.push(ChatMessage::user(refinement));

  let chat_req = ChatRequest::new(messages.clone());

  let chat_res =
    client.exec_chat(model, chat_req, None).await.context("Failed to execute refine request")?;

  let response_text = chat_res.first_text().context("No refined command from AI")?;

  Ok((response_text.trim().to_string(), messages))
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

fn detect_shell() -> String {
  std::env::var("SHELL")
    .ok()
    .and_then(|path| {
      std::path::Path::new(&path).file_name().and_then(|name| name.to_str()).map(String::from)
    })
    .unwrap_or_else(|| {
      eprintln!("Could not detect shell from SHELL environment variable.");
      eprintln!("Defaulting to bash.");
      "bash".to_string()
    })
}

fn print_bash_integration(alias_name: &str) {
  println!(
    r#"# tinyassist shell integration for bash
# Add this to your ~/.bashrc or ~/.bash_profile:
# eval "$(tinyassist --init)"

function {alias}() {{
    export TA_EXIT_CODE=$?
    export TA_SHELL=bash
    export TA_LAST_COMMAND=$(fc -ln -1 2>/dev/null)

    tinyassist "$@"

    unset TA_SHELL
    unset TA_EXIT_CODE
    unset TA_LAST_COMMAND
}}"#,
    alias = alias_name
  );
}

/// Print shell integration script for the specified shell
fn print_shell_integration(alias_name: &str) -> Result<()> {
  let shell = detect_shell();

  if shell == "bash" {
    print_bash_integration(alias_name);
    Ok(())
  } else {
    anyhow::bail!("Unsupported shell: {}. Supported shells: bash", shell);
  }
}

async fn run_user_action(
  action: UserAction,
  command: &str,
  messages: &[ChatMessage],
  model: &str,
) -> Result<(i32, Option<(String, Vec<ChatMessage>)>)> {
  match action {
    UserAction::Copy => {
      copy_to_clipboard(command).await?;
      Ok((0, None))
    }
    UserAction::Run => {
      let exit_code = execute_command(command).await?;
      Ok((exit_code, None))
    }
    UserAction::Explain => {
      let explanation = explain_command(model, command).await?;
      println!("\n{}", "Explanation:".green());
      println!("{}\n", explanation);
      Ok((0, None))
    }
    UserAction::Refine => {
      let refinement = prompt("Instructions: ").await?;
      if refinement.trim().is_empty() {
        println!("{}", "No refinement provided.".yellow());
        Ok((0, None))
      } else {
        let refined_result = refine_command(model, messages.to_vec(), &refinement).await?;
        Ok((0, Some(refined_result)))
      }
    }
    UserAction::Abort => Ok((1, None)),
  }
}

/// Interactive command loop that handles user actions
async fn command_interaction_loop(
  initial_command: String,
  initial_messages: Vec<ChatMessage>,
  model: &str,
) -> Result<i32> {
  let mut current_command = initial_command;
  let mut current_messages = initial_messages;

  loop {
    print_response(&current_command);

    let action = UserAction::ask().await?;
    let (exit_code, new_result) =
      run_user_action(action, &current_command, &current_messages, model).await?;

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
