mod terminal;

use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use directories::ProjectDirs;
use genai::Client;
use genai::chat::{ChatMessage, ChatRequest};
use terminal::{UserAction, copy_to_clipboard, error, execute_command, get_user_action, get_user_input};

/// CLI Arguments
#[derive(Parser, Debug)]
#[command(name = "tinyassist")]
#[command(about = "Generate shell commands from natural language descriptions")]
#[command(version = "0.1.0")]
pub struct Args {
    /// Model to use
    #[arg(long, default_value = "gpt-4o-mini")]
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
    fn get_prompt(&self) -> String {
        self.prompt.join(" ")
    }
}

// AI command generation
const SYSTEM_PROMPT: &str = r#"You are an AI designed to help users identify the precise shell command they need.
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

async fn fix_last_command(
    model: &str,
    context: &terminal::context::TerminalContext,
) -> Result<(String, Vec<ChatMessage>)> {
    let client = Client::default();

    let contextual_prompt = context.format_for_llm();

    let messages = vec![
        ChatMessage::system(FIX_SYSTEM_PROMPT),
        ChatMessage::user(&contextual_prompt),
    ];

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
    context: &terminal::context::TerminalContext,
) -> Result<(String, Vec<ChatMessage>)> {
    let client = Client::default();

    let contextual_prompt = format!(
        "{}\n\nContext:\nWorking directory: {}\nShell: {}",
        user_prompt, context.working_directory, context.shell
    );

    let messages = vec![
        ChatMessage::system(SYSTEM_PROMPT),
        ChatMessage::user(&contextual_prompt),
    ];
    let chat_req = ChatRequest::new(messages.clone());

    let chat_res = client
        .exec_chat(model, chat_req, None)
        .await
        .context("Failed to execute chat request")?;

    let response_text = chat_res.first_text().context("No response text from AI")?;

    Ok((response_text.trim().to_string(), messages))
}

async fn explain_command(model: &str, command: &str) -> Result<String> {
    let client = Client::default();

    let explain_prompt = format!("Explain this shell command in 1-2 simple sentences:\n\n{}", command);

    let chat_req = ChatRequest::new(vec![
        ChatMessage::system(
            "You are a helpful AI that explains shell commands very concisely. Keep it under 50 words.",
        ),
        ChatMessage::user(&explain_prompt),
    ]);

    let chat_res = client
        .exec_chat(model, chat_req, None)
        .await
        .context("Failed to execute explain request")?;

    let response_text = chat_res.first_text().context("No explanation text from AI")?;

    Ok(response_text.trim().to_string())
}

async fn refine_command(
    model: &str,
    original_messages: &[ChatMessage],
    refinement: &str,
) -> Result<(String, Vec<ChatMessage>)> {
    let client = Client::default();

    let mut messages = original_messages.to_vec();
    messages.push(ChatMessage::user(refinement));

    let chat_req = ChatRequest::new(messages.clone());

    let chat_res = client
        .exec_chat(model, chat_req, None)
        .await
        .context("Failed to execute refine request")?;

    let response_text = chat_res.first_text().context("No refined command from AI")?;

    Ok((response_text.trim().to_string(), messages))
}

/// Load .env file if it exists
fn load_env() {
    if let Some(config_path) = get_config_dir() {
        let env_file = config_path.join(".env");
        if dotenvy::from_path(&env_file).is_ok() {
            return;
        }
    }

    let _ = dotenvy::dotenv();
}

/// Get the configuration directory for tinyassist
fn get_config_dir() -> Option<std::path::PathBuf> {
    ProjectDirs::from("", "", "tinyassist").map(|proj_dirs| proj_dirs.config_dir().to_path_buf())
}

fn print_response(command: &str) {
    println!("{} {}", ">".blue(), command);
}

/// Print shell integration script for the specified shell
fn print_shell_integration(alias_name: &str) {
    let shell = detect_shell();

    match shell.as_str() {
        "bash" => print_bash_integration(alias_name),
        _ => {
            eprintln!("Unsupported shell: {}. Supported shells: bash", shell);
            eprintln!("Set SHELL environment variable to your shell path.");
            std::process::exit(1);
        }
    }
}

fn detect_shell() -> String {
    std::env::var("SHELL")
        .ok()
        .and_then(|path| {
            std::path::Path::new(&path)
                .file_name()
                .and_then(|name| name.to_str())
                .map(String::from)
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

        let action = get_user_action().await?;
        let (exit_code, new_result) =
            run_user_action(action.clone(), &current_command, &current_messages, model).await?;

        match action {
            UserAction::Copy | UserAction::Run => {
                return Ok(exit_code);
            }
            UserAction::Explain => {
                // Continue the loop after showing explanation
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
            let refinement = get_user_input("Instructions: ").await?;
            if refinement.trim().is_empty() {
                println!("{}", "No refinement provided.".yellow());
                Ok((0, None))
            } else {
                let refined_result = refine_command(model, messages, &refinement).await?;
                Ok((0, Some(refined_result)))
            }
        }
        UserAction::Abort => Ok((1, None)),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    load_env();

    let args = Args::parse();

    // Handle --init flag
    if args.init {
        print_shell_integration(args.prompt.first().map(|s| s.as_str()).unwrap_or("ta"));
        std::process::exit(0);
    }

    // Handle --info flag
    if args.info {
        if let Some(config_dir) = get_config_dir() {
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
        std::process::exit(0);
    }

    let exit_code = if args.fix {
        handle_fix_mode(&args.model).await?
    } else {
        handle_normal_mode(&args.model, &args.get_prompt()).await?
    };

    std::process::exit(exit_code);
}

/// Handle --fix mode: fix the last command
async fn handle_fix_mode(model: &str) -> Result<i32> {
    let context = terminal::context::get_terminal_context().context("Failed to get terminal context")?;

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

    let context = terminal::context::get_terminal_context().context("Failed to get terminal context")?;

    let (command, messages) = generate_command(model, prompt, &context).await?;
    command_interaction_loop(command, messages, model).await
}
