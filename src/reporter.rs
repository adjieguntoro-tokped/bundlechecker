use std::sync::atomic::AtomicUsize;

use crate::analyze;
use rayon::prelude::*;

pub struct Report {
  silent: bool,
  pub fail: AtomicUsize,
  pub success: AtomicUsize,
  pub error: AtomicUsize,
  pub total: usize,
}

impl Report {
  pub fn new(silent: bool) -> Self {
    Report {
      silent,
      fail: AtomicUsize::new(0),
      success: AtomicUsize::new(0),
      error: AtomicUsize::new(0),
      total: 0,
    }
  }

  pub fn report(
    &mut self,
    analyze_result: &fxhash::FxHashMap<String, analyze::AnalyzeReport>,
  ) -> &Self {
    self.total = analyze_result.len();

    analyze_result.par_iter().for_each(|r| {
      let (file_name, report) = r;
      if report.pass {
        self
          .success
          .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        if !self.silent {
          println!(
            "PASS {file_name}: {} {} < maxSize {} {} ({})",
            report.actual_file_size,
            report.size_unit,
            report.budget_size,
            report.size_unit,
            report.compression
          )
        }
      } else {
        if report.error.is_some() {
          self.error.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
          if !self.silent {
            println!("ERROR {}", report.error.as_ref().unwrap());
          }
        } else {
          if !self.silent {
            println!(
              "FAIL {file_name}: {} {} > maxSize {} {} ({})",
              report.actual_file_size,
              report.size_unit,
              report.budget_size,
              report.size_unit,
              report.compression
            )
          }
          self.fail.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
      }
    });
    self
  }
}
