use std::{collections::HashMap, env, io::{BufRead, BufReader}, process::{Command, Stdio}, sync::{atomic::{AtomicUsize, Ordering}, Arc}, thread::{self, JoinHandle}};
use anyhow::{anyhow, Result};
use regex::Regex;
use serde::Deserialize;
use crate::puff::Puff;

use super::{shell::Shell, task::Task};

/// # Example
/// ```yaml
/// commands:
///   - thread: gcc main.cpp
/// ```
#[derive(Deserialize, Clone, Debug)]
pub struct Threaded {
  thread: String
}

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum TaskCommand {
  Classic(String),
  Multithreaded(Threaded)
}

impl TaskCommand {
  /// Finds variable value
  pub fn find_var(
    &self,
    name: &str,
    args: &Option<Vec<String>>,
    task: &Task,
    puff: &Puff
  ) -> Result<String> {
    if let Some(arguments) = &task.arguments {
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
    task: &Task,
    puff: &Puff,
  ) -> Result<String> {
    let reg = Regex::new(r"\$\{([^}]+)\}")?;
    let mut values = HashMap::new();

    for cap in reg.captures_iter(&cmd) {
      if let Some(var) = cap.get(1) {
        let key = var.as_str().to_string();
        if !values.contains_key(&key) {
          values.insert(key.clone(), self.find_var(&key, args, task, puff)?);
        }
      }
    }

    let result = values.iter().fold(cmd, |acc, (var, value)| {
      acc.replace(&format!("${{{var}}}"), value)
    });

    Ok(result)
  }

  pub fn execute(
    &self,
    shell: &Shell,
    args: Option<Vec<String>>,
    task: (&Task, usize),
    puff: &Puff,
    counter: Arc<AtomicUsize>
  ) -> Result<(i32, Option<JoinHandle<()>>)> {
    match self {
      Self::Classic(cmd) => {
        let cmd = Command::new(&shell.0)
          .arg(shell.get_command_arg())
          .arg(cmd)
          .status()?;

        Ok((cmd.code().unwrap_or_default(), None))
      },
      Self::Multithreaded(threaded) => {
        let (task, task_id) = task;
        let command = self.format(threaded.thread.clone(), &args, task, puff)?;
        let shell_cmd = shell.0.clone();
        let shell_arg = shell.get_command_arg();

        let handle = thread::spawn(move || {
          // println!("thread spawned for task {task_id}");

          let mut process = Command::new(shell_cmd)
            .arg(shell_arg)
            .arg(command)
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to spawn process");

          if let Some(stdout) = process.stdout.take() {
            let reader = BufReader::new(stdout);

            for line in reader.lines().map_while(Result::ok) {
              println!("[#{task_id}] {}", line);
            }
          }

          let _ = process.wait();
          counter.fetch_sub(1, Ordering::SeqCst);
        });

        Ok((0, Some(handle)))
      }
    }
  }
}