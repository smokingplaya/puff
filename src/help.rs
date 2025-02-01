use colored::Colorize;
use crate::puff::Puff;

#[allow(unused)]
pub(crate) fn help(
  puff: Puff,
  file_name: String
) {
  println!(
    "🐧 {} v{} {}",
    "puff".green().bold(),
    env!("CARGO_PKG_VERSION").bright_green(),
    "https://github.com/smokingplaya/puff".bright_black().underline()
  );

  // println!();

  println!("   {} - {}", "help".green(), "Displays a list of available commands".bright_black());
  println!("   {} - {}", "list".green(), "Displays a list of available tasks".bright_black());
}