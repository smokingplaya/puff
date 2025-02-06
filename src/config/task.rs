use std::{collections::HashMap, env, process::Command};
use anyhow::{anyhow, Result};
use regex::Regex;
use crate::{config::shell::Shell, puff::Puff};
use super::argument::Argument;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct Task {
  pub description: Option<String>,
  pub arguments: Option<Vec<Argument>>,
  pub commands: Vec<String>
}

impl Task {
  pub fn is_default(
    &self,
    name: &String,
    puff: &Puff
  ) -> bool {
    *name == puff.get_default_task()
  }

  /// Finds variable value
  pub fn find_var(
    &self,
    name: &str,
    args: &Option<Vec<String>>,
    puff: &Puff
  ) -> Result<String> {
    if let Some(arguments) = &self.arguments {
      if let Some(args) = args {
        if let Some((ind, arg_data)) = arguments.iter().enumerate().find(|(_, arg)| arg.name == name) {
          return args.get(ind)
            .cloned()
            .or_else(|| arg_data.default.clone())
            .ok_or_else(|| anyhow!("argument #{ind} ({name}) must be provided!"));
        }
      }
    }

    puff.variables
      .as_ref()
      .and_then(|vars| vars.get(name).cloned())
      .or_else(|| env::var(name).ok())
      .ok_or_else(|| anyhow!("argument {name} not found!"))
  }

  /// Formats given command
  pub fn format(
    &self,
    cmd: String,
    args: &Option<Vec<String>>,
    puff: &Puff,
  ) -> Result<String> {
    let reg = Regex::new(r"\$\{([^}]+)\}")?;
    let mut values = HashMap::new();

    for cap in reg.captures_iter(&cmd) {
      if let Some(var) = cap.get(1) {
        let key = var.as_str().to_string();
        if !values.contains_key(&key) {
          values.insert(key.clone(), self.find_var(&key, args, puff)?);
        }
      }
    }

    let result = values.iter().fold(cmd, |acc, (var, value)| {
      acc.replace(&format!("${{{var}}}"), value)
    });

    Ok(result)
  }

  pub fn run(
    &self,
    puff: &Puff,
    args: Option<Vec<String>>
  ) -> Result<()> {
    let shell = puff.configuration
      .clone()
      .unwrap_or_default()
      .shell
      .unwrap_or(Shell::default());

    let commands: Vec<String> = self.commands.iter()
      .map(|cmd| self.format(cmd.to_owned(), &args, puff))
      .collect::<Result<_>>()?;

    Command::new(&shell.0)
      .arg(shell.get_command_arg())
      .arg(commands.join(";"))
      .status()?;

    Ok(())
  }
}