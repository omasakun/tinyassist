use indoc::indoc;

use crate::error::{AppError, Result};
use crate::platform::Shell;

pub fn print_shell_integration(shell_name: &str, alias_name: &str) -> Result<()> {
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
