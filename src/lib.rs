#![deny(clippy::all)]

use std::process;

use rayon::prelude::*;

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
  pub reporter: String,
}

#[napi]
pub fn check_bundler_sync(input: CheckBundlerInput) {
  let compression = files::get_file_compression(&input.compression);
  let config = config::get_config(&input.config_path);

  let bundle_files = files::Files::new(config.bundlesize, compression).collect();

  match bundle_files {
    Ok(v) => {
      let reporter = reporter::get_reporter(&input.reporter);
      let result = analyze::Analyzer::new(v).analyze();

      if matches!(reporter, reporter::Reporter::StandardOutput) {
        result.par_iter().for_each(|r| {
          let (file_name, result) = r;
          if result.pass {
            println!(
              "PASS {file_name}: {} {} < maxSize {} {} ({})",
              result.actual_file_size,
              result.size_unit,
              result.budget_size,
              result.size_unit,
              result.compression
            )
          } else {
            if result.error.is_some() {
              println!("ERROR {}", result.error.as_ref().unwrap());
            } else {
              println!(
                "PASS {file_name}: {} {} > maxSize {} {} ({})",
                result.actual_file_size,
                result.size_unit,
                result.budget_size,
                result.size_unit,
                result.compression
              )
            }
          }
        })
      }
    }
    Err(e) => {
      eprintln!("error: {e}");
      eprintln!("program will exit");
      process::exit(1)
    }
  }
}
