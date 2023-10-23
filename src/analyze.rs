use std::sync::{Arc, Mutex};

use crate::files;
use rayon::prelude::*;

#[napi(object)]
#[derive(Debug, Clone)]
pub struct AnalyzeReport {
  pub pass: bool,
  pub actual_file_size: f64,
  pub size_unit: String,
  pub compression: String,
  pub budget_size: f64,
  pub error: Option<String>,
}

pub struct Analyzer {
  pub bundlefiles: fxhash::FxHashMap<String, files::File>,
}

impl Analyzer {
  pub fn new(bundlefiles: fxhash::FxHashMap<String, files::File>) -> Self {
    Analyzer { bundlefiles }
  }

  fn is_budget_pass(&self, actual_file_size: f64, budget_file_size: f64) -> bool {
    if actual_file_size > budget_file_size {
      return false;
    }
    true
  }

  pub fn analyze(&mut self) -> fxhash::FxHashMap<String, AnalyzeReport> {
    let analyze_result = Arc::new(Mutex::new(fxhash::FxHashMap::default()));
    self.bundlefiles.par_iter().for_each(|f| {
      let (file_name, file) = f;
      let file_error = &file.error;
      let size_unit = &file.size_unit;
      let compression = &file.compression;
      if let Some(err) = file_error {
        analyze_result.lock().unwrap().insert(
          file_name.to_string(),
          AnalyzeReport {
            budget_size: file.budget_size,
            actual_file_size: file.actual_file_size,
            size_unit: size_unit.to_string(),
            compression: compression.to_string(),
            pass: false,
            error: Some(err.to_string()),
          },
        );
      } else {
        analyze_result.lock().unwrap().insert(
          file_name.to_string(),
          AnalyzeReport {
            budget_size: file.budget_size,
            actual_file_size: file.actual_file_size,
            size_unit: size_unit.to_string(),
            compression: compression.to_string(),
            pass: self.is_budget_pass(file.actual_file_size, file.budget_size),
            error: None,
          },
        );
      }
    });

    let result = analyze_result.lock().unwrap();
    result.to_owned()
  }
}
