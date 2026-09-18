use crate::error::{AppError, Result};
use crate::genai::{ChatMessage, ChatRequest, Client, ReasoningEffort};

pub fn generate_command(
  model: &str,
  reasoning_effort: Option<ReasoningEffort>,
  user_prompt: &str,
  context_info: &str,
) -> Result<(String, Vec<ChatMessage>)> {
  let client = Client::new();

  let contextual_prompt = format!("{}\n\n{}", user_prompt, context_info);
  let messages = vec![
    ChatMessage::system(crate::prompts::Prompts::GENERATE),
    ChatMessage::user(&contextual_prompt),
  ];
  let chat_req = ChatRequest::new(messages.clone());

  let chat_res = client.exec_chat(model, reasoning_effort, chat_req)?;

  let response_text = chat_res
    .first_text()
    .ok_or_else(|| AppError::Context("No response text from AI".to_string()))?;

  let command = response_text.trim().to_string();
  let mut messages_with_response = messages;
  messages_with_response.push(ChatMessage::assistant(&command));

  Ok((command, messages_with_response))
}

pub fn explain_command(
  model: &str,
  reasoning_effort: Option<ReasoningEffort>,
  command: &str,
) -> Result<String> {
  let client = Client::new();

  let explain_prompt =
    format!("Explain this shell command in 1-2 simple sentences:\n\n{}", command);

  let chat_req = ChatRequest::new(vec![
    ChatMessage::system(crate::prompts::Prompts::EXPLAIN),
    ChatMessage::user(&explain_prompt),
  ]);

  let chat_res = client.exec_chat(model, reasoning_effort, chat_req)?;

  let response_text = chat_res
    .first_text()
    .ok_or_else(|| AppError::Context("No explanation text from AI".to_string()))?;

  Ok(response_text.trim().to_string())
}

pub fn refine_command(
  model: &str,
  reasoning_effort: Option<ReasoningEffort>,
  mut messages: Vec<ChatMessage>,
  refinement: &str,
) -> Result<(String, Vec<ChatMessage>)> {
  let client = Client::new();

  messages.push(ChatMessage::user(refinement));

  let chat_req = ChatRequest::new(messages.clone());

  let chat_res = client.exec_chat(model, reasoning_effort, chat_req)?;

  let response_text = chat_res
    .first_text()
    .ok_or_else(|| AppError::Context("No response text from AI".to_string()))?;

  let command = response_text.trim().to_string();
  messages.push(ChatMessage::assistant(&command));

  Ok((command, messages))
}
