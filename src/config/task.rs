use std::{collections::HashMap, env, process::Command};
use anyhow::{Result, anyhow};
use regex::Regex;
use crate::puff::Puff;
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

    for var in reg.captures_iter(&cmd).map(|c| c[1].to_string()) {
      values.entry(var.clone()).or_insert_with(|| self.find_var(&var, args, puff).unwrap_or_default());
    }

    Ok(values.iter().fold(cmd, |acc, (var, value)| acc.replace(&format!("${{{var}}}"), value)))
  }


  pub fn run(
    &self,
    puff: &Puff,
    args: Option<Vec<String>>
  ) -> Result<()> {
    let shell = puff.configuration
      .clone()
      .ok_or_else(|| anyhow!("shell not found in configuration"))?
      .shell
      .unwrap_or_default();

    let commands: Vec<String> = self.commands.iter()
      .map(|cmd| self.format(cmd.to_owned(), &args, puff))
      .collect::<Result<_>>()?;

    println!("{commands:?}");

    Command::new(&shell.0)
      .arg(shell.get_command_arg())
      .arg(commands.join(";"))
      .status()?;

    Ok(())
  }
}