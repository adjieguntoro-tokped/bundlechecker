use serde::Deserialize;
use std::{fs::File, io::BufReader};

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
  let pkg_json_file = File::open(path).expect("file cannot be open");
  let reader = BufReader::new(pkg_json_file);
  let config: ConfigFile = serde_json::from_reader(reader).expect("cannot serialze");
  config
}
