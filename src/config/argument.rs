use std::fmt::Write;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct Argument {
  /// Name of a argument
  pub name: String,
  /// Default value (if not provided)
  pub default: Option<String>,
  /// Possible values of argument
  pub options: Option<Vec<String>>
}

pub trait CustomToString {
  fn to_string(&self) -> String;
}

impl CustomToString for Vec<Argument> {
  fn to_string(&self) -> String {
    let mut result = String::new();

    for arg in self {
      write!(
        &mut result,
        " <{}{}>",
        arg.name,
        arg.default.as_ref().map_or_else(String::new, |d| format!("={}", d))
      ).unwrap();
    }

    result
  }
}