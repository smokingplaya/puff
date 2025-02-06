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

const PUFF_FILES: [&str; 3] = ["puff.yaml", "puff.yml", ".puff"];

impl Puff {
  fn check_puff_file(
    path: String
  ) -> Result<Puff> {
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
  pub fn list(
    &self
  ) -> Result<()> {
    let data = self.clone()
      .data
     .unwrap_or_default();

    let authors_list = data.authors.unwrap_or_default();
    let authors = if authors_list.is_empty() { "no authors" } else { &("by ".to_owned() + &(authors_list.join(", ")).blue().to_string()) };
    let name = if data.project_name.is_empty() { "no project name" } else {&data.project_name};

    println!("🔥 {}", name.cyan().bold());
    println!("   {authors}");

    if let Some(description) = data.description {
      println!();
      println!("   {description}");
    }

    println!();

    self.tasks.iter()
      .for_each(|(name, data)| {
        let default = data.is_default(name, self);

        println!(
          "   {} {}",
          if default {"•"} else {"○"},
          if default {name.blue().bold()} else {name.bright_blue()}
        );

        println!(
          "     puff {name}{} - {}",
          data.arguments.clone().unwrap_or_default().to_string(),
          data.description.clone().unwrap_or(if default {"default task".to_string()} else {"no description".to_string()})
        );
      });

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
