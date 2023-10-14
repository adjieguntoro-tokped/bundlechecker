use std::{os::unix::prelude::MetadataExt, path::Path};

use anyhow::Result;
use fxhash::FxHashMap;

use crate::files::{FileCompression, FileUnit};

#[derive(Debug)]
pub struct AnalyzeResult {
  pass: bool,
  f_size: f64,
  budget_size: f64,
}

pub struct Analyzer {
  pub compression: FileCompression,
  pub f_size_map: FxHashMap<String, AnalyzeResult>,
}

impl Analyzer {
  pub fn new(compression: FileCompression) -> Self {
    Analyzer {
      compression,
      f_size_map: Default::default(),
    }
  }

  fn analyze_glob(&mut self, path: &String, unit: &Option<FileUnit>) -> Result<()> {
    let mut file_walker = globwalk::glob(path)?.peekable();
    if file_walker.peek().is_none() {
      eprintln!("ERROR: {path} pattern is not getting any match");
    }

    for f in file_walker {
      let dir_entry = f?;
      let f_meta = dir_entry.metadata()?;
      if f_meta.is_file() {
        let f_name = dir_entry.file_name().to_os_string().into_string().unwrap();
        self.analyze_file_size(f_name, f_meta.size(), &unit)
      }
    }
    Ok(())
  }

  fn analyze_single_file(&mut self, path: &String, unit: &Option<FileUnit>) -> Result<()> {
    let f = Path::new(&path);
    let f_name = f.file_name().unwrap().to_os_string().into_string().unwrap();
    let f_meta = f.metadata().expect("cannot extract metadata");
    self.analyze_file_size(f_name, f_meta.size(), &unit);
    Ok(())
  }

  pub fn analyze(&mut self, path: String, unit: Option<FileUnit>) -> Result<()> {
    let is_glob_path = is_glob::is_glob(&path);
    if is_glob_path {
      self.analyze_glob(&path, &unit)
    } else {
      self.analyze_single_file(&path, &unit)
    }
  }

  fn is_budget_pass(&self, f_size: f64, budget_size: f64) -> bool {
    if budget_size > f_size {
      return true;
    }
    false
  }

  fn analyze_file_size(&mut self, f_name: String, f_size: u64, u: &Option<FileUnit>) {
    match u {
      Some(v) => {
        let conv_unit = v.get_converted_unit(f_size);
        let budget_size = v.get_budget();
        let is_pass = self.is_budget_pass(conv_unit, budget_size);
        self.f_size_map.insert(
          f_name,
          AnalyzeResult {
            pass: is_pass,
            f_size: conv_unit,
            budget_size,
          },
        );
      }
      _ => {}
    }
  }
}
