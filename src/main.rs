use std::env;
use anyhow::Result;
use colored::Colorize;
use puff::Puff;

mod config;
mod help;
mod puff;

fn run() -> Result<()> {
  let mut args = env::args()
    .collect::<Vec<String>>();

  // path to the current executable
  args.remove(0);

  let (puff, file_name) = Puff::find()?;
  let command = args.first();

  match command {
    Some(task) => match task.as_str() {
      "help" => {
        help::help(puff, file_name);
        Ok(())
      },
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
    println!("😡 error: {}", err.to_string().red().bold())
  }
}