use std::fmt::Write;

use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct Argument {
  pub name: String,
  pub default: Option<String>
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