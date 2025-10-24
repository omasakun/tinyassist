use colored::Colorize;

use crate::error::{AppError, Result};
use crate::genai::{ChatMessage, ChatRequest, Client};
use crate::terminal::{ChatAction, copy_to_clipboard, prompt};

pub fn handle_chat_response(
  model: &str,
  client: &Client,
  messages: &mut Vec<ChatMessage>,
) -> Result<String> {
  let chat_req = ChatRequest::new(messages.clone());
  let chat_res = client.exec_chat(model, chat_req)?;

  let response_text = chat_res
    .first_text()
    .ok_or_else(|| AppError::Context("No response text from AI".to_string()))?
    .to_string();

  println!("{} {}", "AI:".green(), response_text);
  messages.push(ChatMessage::assistant(&response_text));

  Ok(response_text)
}

pub fn process_chat_action(
  action: ChatAction,
  response_text: &str,
  messages: &mut Vec<ChatMessage>,
  model: &str,
  client: &Client,
) -> Result<bool> {
  match action {
    ChatAction::Copy => {
      copy_to_clipboard(response_text)?;
      println!("{}", "Copied to clipboard".green());
      Ok(true)
    }
    ChatAction::Revert => {
      messages.pop();
      messages.pop();
      if messages.len() > 1 {
        println!("{}", "Reverted. You can enter a different prompt".yellow());
        let corrected_input = prompt("You: ")?;

        if !corrected_input.trim().is_empty() {
          messages.push(ChatMessage::user(&corrected_input));

          let new_response_text = handle_chat_response(model, client, messages)?;

          messages.pop();
          messages.push(ChatMessage::assistant(&new_response_text));
        }
      }
      Ok(false)
    }
    ChatAction::Next => Ok(true),
    ChatAction::Abort => {
      // Signal abort by returning None through caller
      Ok(true)
    }
  }
}
