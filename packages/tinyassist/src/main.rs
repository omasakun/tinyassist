mod terminal;

use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use genai::Client;
use genai::chat::{ChatMessage, ChatRequest};
use terminal::{UserAction, copy_to_clipboard, error, execute_command, get_user_action, get_user_input};

// CLI Arguments
#[derive(Parser, Debug)]
#[command(name = "tinyassist")]
#[command(about = "Generate shell commands from natural language descriptions")]
#[command(version = "0.1.0")]
pub struct Args {
    /// Model to use (e.g., gpt-4o-mini, claude-3-haiku-20240307)
    #[arg(long, default_value = "gpt-4o-mini")]
    pub model: String,

    /// Natural language description of the command you want
    #[arg(required = true)]
    pub prompt: Vec<String>,
}

impl Args {
    pub fn get_prompt(&self) -> String {
        self.prompt.join(" ")
    }
}

// AI command generation
const SYSTEM_PROMPT: &str = r#"You are a helpful AI that helps user identify the shell command they are looking for.
Always provide a concise shell command.

Important:
- Think hard about the command you are going to provide, and make sure it is the best one.
- No shebang
- No explanations
- No extra text. Just the command.

User uses Linux/macOS"#;

async fn generate_command(model: &str, user_prompt: &str) -> Result<(String, Vec<ChatMessage>)> {
    let client = Client::default();

    let messages = vec![ChatMessage::system(SYSTEM_PROMPT), ChatMessage::user(user_prompt)];
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

/// Load .env file if it exists (silent failure is OK)
fn load_env() {
    let _ = dotenvy::dotenv();
}

fn print_response(command: &str) {
    println!("{} {}", ">".blue(), command);
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
    let args = Args::parse();

    // Load environment variables
    load_env();

    let prompt = args.get_prompt();
    if prompt.trim().is_empty() {
        error("Please provide a prompt");
        std::process::exit(1);
    }

    let (mut current_command, mut current_messages) = generate_command(&args.model, &prompt).await?;

    loop {
        print_response(&current_command);

        let action = get_user_action().await?;
        let (exit_code, new_result) =
            run_user_action(action.clone(), &current_command, &current_messages, &args.model).await?;

        match action {
            UserAction::Copy | UserAction::Run => {
                std::process::exit(exit_code);
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
                std::process::exit(1);
            }
        }
    }
}
