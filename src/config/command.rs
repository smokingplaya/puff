use std::{collections::HashMap, env, io::{BufRead, BufReader}, process::{Command, Stdio}, sync::{atomic::{AtomicUsize, Ordering}, Arc}, thread::{self, JoinHandle}, time::Duration};
use anyhow::{anyhow, Result};
use colored::Colorize;
use regex::Regex;
use serde::Deserialize;
use crate::puff::Puff;

use super::{shell::Shell, task::Task};

/// # Example
/// ```yaml
/// commands:
///   - async: gcc main.cpp
/// ```
#[derive(Deserialize, Clone, Debug)]
pub struct Threaded {
  r#async: String
}

/// # Example
/// ```yaml
/// commands:
///   - async: gcc general.cpp
///   - async: gcc main.cpp
///   - await: echo "builded"
/// ```
#[derive(Deserialize, Clone, Debug)]
pub struct Waiter {
  r#await: String
}

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum TaskCommand {
  Classic(String),          // cmd
  Multithreaded(Threaded),  // async: cmd
  Threadwaiter(Waiter)      // await: cmd
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
    // TODO why it so ugly here
    if let (Some(arguments), Some(args)) = (&task.arguments, args) {
      if let Some((ind, arg_data)) = arguments.iter().enumerate().find(|(_, arg)| arg.name == name) {
        if let Some(arg_value) = args.get(ind).cloned().or_else(|| arg_data.default.clone()) {
          if arg_data.options.as_ref().map_or(true, |opts| opts.contains(&arg_value)) {
            return Ok(arg_value);
          }
          return Err(anyhow!("argument #{ind} ({name}) has an invalid value!"));
        }
        return Err(anyhow!("argument #{ind} ({name}) must be provided!"));
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
    let mut result = cmd;

    loop {
      let mut values = HashMap::new();
      let mut found = false;

      for cap in reg.captures_iter(&result) {
        if let Some(var) = cap.get(1) {
          let key = var.as_str().to_string();
          if !values.contains_key(&key) {
            let value = self.find_var(&key, args, task, puff)?;
            values.insert(key.clone(), value);
            found = true;
          }
        }
      }

      if !found {
        break;
      }

      result = values.iter().fold(result, |acc, (var, value)| {
        acc.replace(&format!("${{{var}}}"), value)
      });
    }

    Ok(result)
  }

  pub fn execute(
    &self,
    shell: &Shell,
    args: Option<Vec<String>>,
    task: (&Task, usize),
    puff: &Puff,
    thread_count: Arc<AtomicUsize>
  ) -> Result<(i32, Option<JoinHandle<()>>)> {
    match self {
      Self::Classic(command) => self.execute_classic(shell, args, task, puff, command),
      Self::Threadwaiter(waiter) => self.execute_threadwaiter(shell, args, task, puff, waiter.r#await.clone(), thread_count),
      Self::Multithreaded(threaded) => self.execute_multithreaded(shell, args, task, puff, thread_count, threaded.r#async.clone()),
    }
  }

  fn execute_classic(
    &self,
    shell: &Shell,
    args: Option<Vec<String>>,
    task: (&Task, usize),
    puff: &Puff,
    command: &String
  ) -> Result<(i32, Option<JoinHandle<()>>)> {
    let (task, _) = task;
    let formatted_cmd = self.format(command.to_owned(), &args, task, puff)?;
    let status = Command::new(&shell.0)
      .arg(shell.get_command_arg())
      .arg(formatted_cmd)
      .status()?;

    Ok((status.code().unwrap_or_default(), None))
  }

  fn execute_threadwaiter(
    &self,
    shell: &Shell,
    args: Option<Vec<String>>,
    task: (&Task, usize),
    puff: &Puff,
    command: String,
    thread_count: Arc<AtomicUsize>
  ) -> Result<(i32, Option<JoinHandle<()>>)> {
    let (task, _) = task;
    let formatted_cmd = self.format(command, &args, task, puff)?;

    while thread_count.load(Ordering::SeqCst) > 0 {
      thread::sleep(Duration::from_millis(100));
    }

    let status = Command::new(&shell.0)
      .arg(shell.get_command_arg())
      .arg(formatted_cmd)
      .status()?;
    Ok((status.code().unwrap_or_default(), None))
  }

  fn execute_multithreaded(
    &self,
    shell: &Shell,
    args: Option<Vec<String>>,
    task: (&Task, usize),
    puff: &Puff,
    thread_count: Arc<AtomicUsize>,
    command: String
  ) -> Result<(i32, Option<JoinHandle<()>>)> {
    let (task, task_id) = task;
    let command = self.format(command, &args, task, puff)?;
    let shell_cmd = shell.0.clone();
    let shell_arg = shell.get_command_arg();

    let show_logs = puff.configuration
      .clone()
      .unwrap_or_default()
      .show_thread_logs
      .unwrap_or_default();

    let handle = thread::spawn(move || {
      let mut process = Command::new(shell_cmd)
        .arg(shell_arg)
        .arg(command)
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn process");

      // todo - ugly
      if show_logs {
        if let Some(stdout) = process.stdout.take() {
          let reader = BufReader::new(stdout);
          for line in reader.lines().map_while(Result::ok) {
            println!("{} {}", format!("[#{task_id}]").blue().bold(), line.italic());
          }
        }
      }

      let _ = process.wait();
      thread_count.fetch_sub(1, Ordering::SeqCst);
    });

    Ok((0, Some(handle)))
  }
}
