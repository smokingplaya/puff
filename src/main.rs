use std::env;
use anyhow::Result;
use puff::Puff;

mod log;
mod config;
mod help;
mod puff;

fn run() -> Result<()> {
  let mut args = env::args()
    .collect::<Vec<String>>();

  // path to the current executable
  args.remove(0);

  let command = args.first();

  if let Some(cmd) = command {
    if cmd == "help" {
      return {
        help::help();
        Ok(())
      };
    }
  }

  let puff = Puff::find()?;

  match command {
    Some(task) => match task.as_str() {
      "list" => puff.list(),
      _ => puff.run(Some(task.to_owned()), {
        args.remove(0);
        Some(args)
      }),
    },
    None => puff.run(None, None)
  }
}

fn main() {
  if let Err(err) = run() {
    log::error(err)
  }
}