use colored::Colorize;

pub(crate) fn error<T: ToString>(text: T) {
  println!("{} {}", "[error]".red().bold(), text.to_string().italic());
}