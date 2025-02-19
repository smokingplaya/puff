use std::{process, sync::{atomic::{AtomicUsize, Ordering}, Arc}, thread, time::Duration};
use anyhow::Result;
use crate::{log, puff::Puff};
use super::{argument::Argument, command::TaskCommand};
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct Task {
  pub description: Option<String>,
  pub arguments: Option<Vec<Argument>>,
  pub commands: Vec<TaskCommand>
}

impl Task {
  pub fn is_default(
    &self,
    name: &String,
    puff: &Puff
  ) -> bool {
    *name == puff.get_default_task()
  }

  pub fn run(
    &self,
    puff: &Puff,
    args: Option<Vec<String>>
  ) -> Result<()> {
    let thread_counter = Arc::new(AtomicUsize::new(0));

    let shell = puff.configuration
      .clone()
      .unwrap_or_default()
      .shell
      .unwrap_or_default();

    // todo validate arguments before running commands

    for (ind, command) in self.commands.iter().enumerate() {
      let (exit_code, handle) = command.execute(
        &shell,
        args.clone(),
        (self, ind),
        puff,
        Arc::clone(&thread_counter)
      )?;

      if exit_code != 0 {
        log::error(format!("task {ind} exited with status {exit_code}"));
        process::exit(exit_code);
      }

      if handle.is_some() {
        let counter = Arc::clone(&thread_counter);
        counter.fetch_add(1, Ordering::SeqCst);
      }
    }

    // yep
    while thread_counter.load(Ordering::SeqCst) > 0 {
      thread::sleep(Duration::from_millis(100));
    }

    Ok(())
  }
}