use std::sync::atomic::AtomicUsize;

use crate::analyze;
use rayon::prelude::*;

pub struct Report {
  pub fail: AtomicUsize,
  pub success: AtomicUsize,
  pub error: AtomicUsize,
  pub total: usize,
}

impl Report {
  pub fn new() -> Self {
    Report {
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

        println!(
          "PASS {file_name}: {} {} < maxSize {} {} ({})",
          report.actual_file_size,
          report.size_unit,
          report.budget_size,
          report.size_unit,
          report.compression
        )
      } else {
        if report.error.is_some() {
          self.error.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
          println!("ERROR {}", report.error.as_ref().unwrap());
        } else {
          self.fail.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
          println!(
            "FAIL {file_name}: {} {} > maxSize {} {} ({})",
            report.actual_file_size,
            report.size_unit,
            report.budget_size,
            report.size_unit,
            report.compression
          )
        }
      }
    });
    self
  }
}
