use serde::Deserialize;
use super::shell::Shell;

#[derive(Deserialize, Clone)]
pub struct DefaultTask(pub String);

impl Default for DefaultTask {
  fn default() -> Self {
    Self(String::from("default"))
  }
}

#[derive(Deserialize, Default, Clone)]
pub struct Configuration {
  /// Override runtime shell
  pub shell: Option<Shell>,
  /// Default task
  pub default: Option<DefaultTask>
}