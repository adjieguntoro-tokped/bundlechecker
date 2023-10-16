use crate::files;
use anyhow::Result;

pub enum Reporter {
  StandardOutput,
}

#[derive(Debug, Clone)]
pub struct AnalyzeReport {
  pub pass: bool,
  pub file_name: String,
  pub actual_file_size: f64,
  pub budget_size: f64,
  pub error: Option<String>,
}

pub struct Analyzer {
  reporter: Reporter,
  pub bundlefiles: fxhash::FxHashMap<String, files::File>,
  report: fxhash::FxHashMap<String, AnalyzeReport>,
}

pub fn get_reporter(reporter: &str) -> Reporter {
  match reporter {
    "stdout" => Reporter::StandardOutput,
    _ => Reporter::StandardOutput,
  }
}

impl Analyzer {
  pub fn new(reporter: Reporter, bundlefiles: fxhash::FxHashMap<String, files::File>) -> Self {
    Analyzer {
      reporter,
      bundlefiles,
      report: Default::default(),
    }
  }

  fn is_budget_pass(&self, actual_file_size: f64, budget_file_size: f64) -> bool {
    if actual_file_size > budget_file_size {
      return false;
    }
    true
  }

  // TODO: analyze shit
  pub fn analze(&mut self) -> Result<()> {
    Ok(())
  }
}
