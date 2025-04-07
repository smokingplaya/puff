use serde::Deserialize;
use std::env;

#[derive(Deserialize, Clone, Debug)]
pub struct Shell(pub String);

impl Shell {
  fn get_current_shell() -> String {
    if cfg!(target_os = "windows") {
      // cmd by default
      env::var("COMSPEC")
        .unwrap_or("cmd".to_string())
    } else {
      env::var("SHELL")
        .unwrap_or("sh".to_string())
    }
  }

  pub fn get_command_arg(
    &self
  ) -> String {
    // split this shit 'cuz we can got /usr/bin/zsh for example, not only zsh/bash/etc
    let cmd = self.0
      .replace("\\", "/") // for windows
      .replace(".exe", "") // for windows
      .split("/")
      .last()
      .unwrap()
      .to_string();

    match cmd.as_str() {
      "cmd" => String::from("/C"),
      "powershell" => String::from("/Command"),
      _ => String::from("-c") // dk
    }
  }
}

impl Default for Shell {
  fn default() -> Self {
    Shell(Self::get_current_shell())
  }
}