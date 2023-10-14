use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleConfig {
  pub path: String,
  pub max_size: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigFile {
  pub bundlesize: Vec<BundleConfig>,
}

pub fn get_config(path: &str) -> ConfigFile {
  let json_str = fs::read_to_string(path).expect("file cannot be open");
  let config: ConfigFile = serde_json::from_str(&json_str).expect("cannot serialze");
  config
}
