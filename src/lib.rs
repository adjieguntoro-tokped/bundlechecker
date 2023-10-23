#![deny(clippy::all)]

use std::{process, sync::atomic::Ordering};

use napi::Result;

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

#[napi(object)]
pub struct BundleOutputSummary {
  pub total: u32,
  pub success: u32,
  pub fail: u32,
  pub error: u32,
}

#[napi(object)]
pub struct CheckBundlerOutput {
  pub summary: BundleOutputSummary,
  pub result: fxhash::FxHashMap<String, analyze::AnalyzeReport>,
}

#[napi]
pub fn check_bundler_sync(input: CheckBundlerInput) -> Result<CheckBundlerOutput> {
  let compression = files::get_file_compression(&input.compression);
  let config = config::get_config(&input.config_path);

  let bundle_files = files::Files::new(config.bundlesize, compression).collect();

  match bundle_files {
    Ok(v) => {
      let result = analyze::Analyzer::new(v).analyze();
      let mut reporter = reporter::Report::new();
      let report = reporter.report(&result);

      let total = report.total;
      let success = report.success.load(Ordering::SeqCst);
      let fail = report.fail.load(Ordering::SeqCst);
      let error = report.error.load(Ordering::SeqCst);

      let summary = BundleOutputSummary {
        total: total as u32,
        success: success as u32,
        fail: fail as u32,
        error: error as u32,
      };
      Ok(CheckBundlerOutput { result, summary })
    }
    Err(e) => {
      eprintln!("error: {e}");
      eprintln!("program will exit");
      process::exit(1)
    }
  }
}
