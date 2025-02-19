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
  pub default: Option<DefaultTask>,
  /// Should we shou you logs from multithread command?
  #[serde(default = "logs")]
  pub show_thread_logs: Option<bool>
}

fn logs() -> Option<bool> {
  Some(true)
}