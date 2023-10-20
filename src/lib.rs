#![deny(clippy::all)]

use std::process;

mod analyze;
mod config;
mod files;
mod reporter;

#[macro_use]
extern crate napi_derive;

#[napi(object)]
pub struct CheckBundlerInput {
  pub config_path: String,
  pub compression: String,
}

#[napi]
pub fn check_bundler_sync(input: CheckBundlerInput) {
  let compression = files::get_file_compression(&input.compression);
  let config = config::get_config(&input.config_path);

  let bundle_files = files::Files::new(config.bundlesize, compression).collect();

  match bundle_files {
    Ok(v) => {
      println!("{}", v.len())
    }
    Err(e) => {
      eprintln!("error: {e}");
      eprintln!("program will exit");
      process::exit(1)
    }
  }
}
