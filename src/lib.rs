#![deny(clippy::all)]

use rayon::prelude::*;
use std::{
  process,
  sync::{Arc, Mutex},
};

mod analyze;
mod config;
mod files;

#[macro_use]
extern crate napi_derive;

#[napi(object)]
pub struct CheckBundlerInput {
  pub config_path: String,
  pub compression: String,
}

#[napi]
pub fn check_bundler(input: CheckBundlerInput) {
  let compression = files::get_file_compression(&input.compression);
  let config = config::get_config(&input.config_path);

  println!(
    "Analyzing for config_path={}, compression={:?}",
    &input.config_path, compression
  );

  let analyzer = Arc::new(Mutex::new(analyze::Analyzer::new(compression)));

  config.bundlesize.par_iter().for_each(|c| {
    let path = &c.path;

    let unit = files::get_file_unit(&c.max_size);
    if let None = unit {
      eprintln!("max_size config is not well formatted");
      process::exit(1)
    }

    if let Err(e) = analyzer.lock().unwrap().analyze(path.to_string(), unit) {
      eprintln!("{}", e);
      process::exit(1);
    }
  });

  analyzer
    .lock()
    .unwrap()
    .f_size_map
    .par_iter()
    .for_each(|v| {
      let (f_name, result) = v;
      println!("file_name={}, result={:?}", f_name, result);
    });
}
