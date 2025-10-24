use std::io::{self, Write};
use std::process::Command;
use std::str::FromStr;
use std::thread;
use std::time::Duration;

use arboard::Clipboard;
use colored::Colorize;
use console::{Key, Term};
use strum_macros::{EnumIter, EnumString};

use crate::error::Result;

#[derive(Debug, Clone, Copy, EnumString, EnumIter)]
#[strum(serialize_all = "lowercase")]
pub enum UserAction {
  #[strum(serialize = "c")]
  Copy,
  #[strum(serialize = "r")]
  Run,
  #[strum(serialize = "e")]
  Explain,
  #[strum(serialize = "f")]
  Refine,
  #[strum(serialize = "a")]
  Abort,
}

impl UserAction {
  pub fn ask() -> Result<Self> {
    loop {
      print!("{}", "[c] Copy  [r] Run  [e] Explain  [f] Refine  [a] Abort > ".blue());
      io::stdout().flush()?;

      let key = Term::stderr().read_key()?;
      let c = match key {
        Key::Char(ch) => ch.to_ascii_lowercase(),
        Key::Escape => 'a',
        _ => {
          println!();
          continue;
        }
      };

      println!();

      if let Ok(action) = UserAction::from_str(&c.to_string()) {
        return Ok(action);
      } else {
        println!("{}", "Invalid choice. Please try again.".yellow());
        continue;
      }
    }
  }
}

// Color utilities
pub fn error(message: &str) {
  eprintln!("{}", message.red());
}

// Input and execution utilities
pub fn copy_to_clipboard(text: &str) -> Result<()> {
  let mut clipboard = Clipboard::new()?;
  clipboard.set_text(text)?;

  // Keep clipboard alive for a short duration to ensure clipboard managers
  // have time to detect and store the contents
  thread::sleep(Duration::from_millis(100));
  Ok(())
}

pub fn execute_command(command: &str) -> Result<i32> {
  let status = Command::new("sh").arg("-c").arg(command).status()?;

  Ok(status.code().unwrap_or(1))
}

pub fn prompt(prompt: &str) -> Result<String> {
  print!("{}", prompt.blue());
  io::stdout().flush()?;

  let mut input = String::new();
  io::stdin().read_line(&mut input)?;

  Ok(input.trim().to_string())
}
