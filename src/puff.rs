use std::{collections::HashMap, fs::File};
use crate::config::{argument::CustomToString as _, configuration::Configuration, data::Data, task::Task};
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use serde::Deserialize;

/// Configuration (PuffFile)
#[derive(Deserialize, Clone)]
pub struct Puff {
  pub data: Option<Data>,
  pub configuration: Option<Configuration>,
  pub variables: Option<HashMap<String, String>>,
  pub tasks: HashMap<String, Task>
}

const PUFF_FILES: [&str; 3] = ["puff.yml", "puff.yaml", ".puff"];

impl Puff {
  fn check_puff_file(
    path: String
  ) -> Result<Puff> {
    println!("hi");
    let reader = File::open(path)?;

    Ok(serde_yml::from_reader::<_, Puff>(reader)?)
  }

  /// Finds PuffFile in Current Working Directory
  pub fn find() -> Result<Puff> {
    PUFF_FILES.iter()
      .find_map(|file| Self::check_puff_file(file.to_string()).ok())
      .ok_or_else(|| anyhow!("puff file not found in current directory"))
  }

  pub fn get_default_task(&self) -> String {
    self.configuration.clone()
      .unwrap_or_default()
      .default
      .unwrap_or_default()
      .0
  }

  /// Displays a list of available commands
  pub fn list(&self) -> Result<()> {
    let data = self.clone().data.unwrap_or_default();
    let authors = data.authors.unwrap_or_default();
    let project_name = if data.project_name.is_empty() {
      "no project name".to_string()
    } else {
      data.project_name.clone()
    };

    println!("🔥 {}", project_name.cyan().bold());
    println!("   {}", if authors.is_empty() {
      "no authors".to_string()
    } else {
      format!("by {}", authors.join(", ").blue())
    });

    if let Some(description) = data.description {
      println!("\n   {}", description);
    }
    println!();

    for (name, task) in &self.tasks {
      let is_default = task.is_default(name, self);
      let symbol = if is_default { "•" } else { "○" };
      let formatted_name = if is_default {
        name.blue().bold()
      } else {
        name.bright_blue()
      };
      let args = task.arguments.clone().unwrap_or_default().to_string();
      let desc = task.description.clone().unwrap_or_else(|| {
        if is_default { "default task".to_string() } else { "no description".to_string() }
      });

      println!("   {} {}", symbol, formatted_name);
      println!("     puff {}{} - {}", name, args, desc);
    }

    Ok(())
  }

  pub fn run(
    &self,
    task: Option<String>,
    args: Option<Vec<String>>
  ) -> Result<()> {
    let name = task.unwrap_or(self.get_default_task());
    let task = self.tasks.get(&name)
      .context(anyhow!("task not found"))?;

    task.run(self, args)
  }
}
