use std::io::{self, Write};
use std::process::Command;
use std::thread;
use std::time::Duration;

use anyhow::Result;
use arboard::Clipboard;
use colored::Colorize;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

#[derive(Debug, Clone, Copy)]
pub enum UserAction {
  Copy,
  Run,
  Explain,
  Refine,
  Abort,
}

impl UserAction {
  pub async fn ask() -> Result<Self> {
    loop {
      print!("{}", "[c] Copy  [r] Run  [e] Explain  [f] Refine  [a] Abort > ".blue());
      io::stdout().flush()?;

      enable_raw_mode()?;

      let key = loop {
        if let Event::Key(key_event) = event::read()? {
          match key_event.code {
            KeyCode::Char(c) => break c.to_ascii_lowercase(),
            KeyCode::Esc => break 'a',
            _ => continue,
          }
        }
      };

      disable_raw_mode()?;
      println!();

      match key {
        'c' => return Ok(UserAction::Copy),
        'r' => return Ok(UserAction::Run),
        'e' => return Ok(UserAction::Explain),
        'f' => return Ok(UserAction::Refine),
        'a' => return Ok(UserAction::Abort),
        _ => {
          println!("{}", "Invalid choice. Please try again.".yellow());
          continue;
        }
      }
    }
  }
}

// Color utilities
pub fn error(message: &str) {
  eprintln!("{}", message.red());
}

// Input and execution utilities
pub async fn copy_to_clipboard(text: &str) -> Result<()> {
  let mut clipboard = Clipboard::new()?;
  clipboard.set_text(text)?;

  // Keep clipboard alive for a short duration to ensure clipboard managers
  // have time to detect and store the contents
  thread::sleep(Duration::from_millis(100));
  Ok(())
}

pub async fn execute_command(command: &str) -> Result<i32> {
  let status = Command::new("sh").arg("-c").arg(command).status()?;

  Ok(status.code().unwrap_or(1))
}

pub async fn prompt(prompt: &str) -> Result<String> {
  print!("{}", prompt.blue());
  io::stdout().flush()?;

  let mut input = String::new();
  io::stdin().read_line(&mut input)?;

  Ok(input.trim().to_string())
}
