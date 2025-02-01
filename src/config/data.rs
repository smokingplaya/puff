use serde::Deserialize;

#[derive(Deserialize, Default, Debug, Clone)]
pub struct Data {
  pub authors: Option<Vec<String>>,
  #[serde(rename = "projectName")]
  pub project_name: String,
  pub description: Option<String>,
}