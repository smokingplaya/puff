use colored::Colorize;

pub(crate) fn help() {
  println!(
    "🐧 {} v{} {}",
    "puff".green().bold(),
    env!("CARGO_PKG_VERSION").bright_green(),
    "https://github.com/smokingplaya/puff".bright_black().underline()
  );

  println!("   {} - {}", "help".green(), "Displays a list of available commands".bright_black());
  println!("   {} - {}", "list".green(), "Displays a list of available tasks".bright_black());
}