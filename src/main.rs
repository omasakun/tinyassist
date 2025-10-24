#![allow(clippy::type_complexity)]

mod command_ops;
mod context;
mod error;
mod genai;
mod integrations;
mod platform;
mod prompts;
mod terminal;

use std::path::PathBuf;
use std::process::ExitCode;

use argh::FromArgs;
use colored::Colorize;
use directories::ProjectDirs;
use terminal::{UserAction, copy_to_clipboard, execute_command, prompt};

use crate::context::ShellContext;
use crate::error::{AppError, Result};
use crate::genai::{ChatMessage, ChatRequest, Client};

const DEFAULT_MODEL: &str = "gpt-4o-mini";

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
  #[argh(switch, short = 'c')]
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
    integrations::print_shell_integration(shell_name, alias_name)?;
    return Ok(ExitCode::SUCCESS);
  }

  // Handle --info flag
  if args.info {
    if let Some(config_dir) = Args::config_dir() {
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

  if args.chat {
    let initial_prompt = if args.prompt.is_empty() { None } else { Some(args.prompt()) };
    handle_chat_mode(&args.model, initial_prompt)
  } else if args.fix {
    handle_fix_mode(&args.model, &args.prompt())
  } else {
    let prompt_text = if args.prompt.is_empty() { prompt("Prompt: ")? } else { args.prompt() };
    handle_normal_mode(&args.model, &prompt_text)
  }
}

/// Handle chat mode: free form conversation with LLM
fn handle_chat_mode(model: &str, initial_prompt: Option<String>) -> Result<ExitCode> {
  let client = Client::new();
  let mut messages: Vec<ChatMessage> = Vec::new();

  messages.push(ChatMessage::system(prompts::Prompts::CHAT));

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

    println!("{}  {}", "AI:".green(), response_text);
    messages.push(ChatMessage::assistant(&response_text));
  }
}

/// Handle --fix mode: fix the last command
fn handle_fix_mode(model: &str, prompt: &str) -> Result<ExitCode> {
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
    vec![ChatMessage::system(prompts::Prompts::FIX), ChatMessage::user(&contextual_prompt)];

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
fn handle_normal_mode(model: &str, prompt: &str) -> Result<ExitCode> {
  if prompt.trim().is_empty() {
    return Err(AppError::Context("Please provide a prompt".to_string()));
  }

  let context = ShellContext::current()?;

  let (command, messages) =
    command_ops::generate_command(model, prompt, &context.format_for_llm(false))?;
  command_interaction_loop(command, messages, model)
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

/// Interactive command loop that handles user actions
fn command_interaction_loop(
  initial_command: String,
  initial_messages: Vec<ChatMessage>,
  model: &str,
) -> Result<ExitCode> {
  let mut current_command = initial_command;
  let mut current_messages = initial_messages;

  loop {
    println!("{} {}", ">".blue(), current_command);

    let action = UserAction::ask()?;

    match action {
      UserAction::Copy => {
        copy_to_clipboard(&current_command)?;
        println!("{}", "Copied to clipboard".green());
        return Ok(ExitCode::SUCCESS);
      }
      UserAction::Run => {
        let exit_code = execute_command(&current_command)?;
        return Ok(ExitCode::from(exit_code as u8));
      }
      UserAction::Explain => {
        let explanation = command_ops::explain_command(model, &current_command)?;
        println!("\n{}", "Explanation:".green());
        println!("{}\n", explanation);
      }
      UserAction::Refine => {
        let refinement = prompt("Instructions: ")?;
        if !refinement.trim().is_empty() {
          let (refined_command, refined_messages) =
            command_ops::refine_command(model, current_messages.clone(), &refinement)?;
          current_command = refined_command;
          current_messages = refined_messages;
        } else {
          println!("{}", "No refinement provided.".yellow());
        }
      }
      UserAction::Abort => return Ok(ExitCode::from(1)),
    }
  }
}
