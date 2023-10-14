use std::{
  io::Read,
  os::unix::prelude::MetadataExt,
  path::Path,
  sync::{Arc, Mutex},
};

use anyhow::Result;
use fxhash::FxHashMap;
use rayon::prelude::*;

use crate::files::{FileCompression, FileUnit};

#[napi(object)]
#[derive(Clone, Copy, Debug)]
pub struct AnalyzeResult {
  pub pass: bool,
  pub f_size: f64,
  pub budget_size: f64,
}

pub struct Analyzer {
  compression: FileCompression,
  brotli_enc_params: brotli::enc::BrotliEncoderParams,
  pub f_size_map: Arc<Mutex<FxHashMap<String, AnalyzeResult>>>,
}

impl Analyzer {
  pub fn new(compression: FileCompression) -> Self {
    Analyzer {
      compression,
      brotli_enc_params: brotli::enc::BrotliEncoderParams::default(),
      f_size_map: Default::default(),
    }
  }

  fn get_brotli_size(&self, file_path: &str) -> Result<usize> {
    let f = std::fs::File::open(&file_path)?;
    let mut buf = vec![];
    let mut writer = brotli::CompressorReader::with_params(f, 2048, &self.brotli_enc_params);
    let b_size = writer.read_to_end(&mut buf)?;
    Ok(b_size)
  }

  fn analyze_glob(&self, path: &String, unit: &Option<FileUnit>) -> Result<()> {
    let mut file_walker = globwalk::glob(path)?.peekable();
    if file_walker.peek().is_none() {
      eprintln!("ERROR: {path} pattern is not getting any match");
    }

    let mut v: Vec<globwalk::DirEntry> = vec![];

    for f in file_walker {
      let dir_entry = f?;
      v.push(dir_entry);
    }

    v.par_iter().for_each(|dir_entry| {
      let f_meta = dir_entry.metadata().unwrap();
      if f_meta.is_file() {
        let f_name = dir_entry.file_name().to_os_string().into_string().unwrap();
        match self.compression {
          FileCompression::Brotli => {
            if let Some(p) = dir_entry.path().to_str() {
              let br_size = self.get_brotli_size(p).unwrap();
              self.analyze_file_size(f_name, br_size.try_into().unwrap(), &unit)
            }
          }
          FileCompression::UnCompressed => self.analyze_file_size(f_name, f_meta.size(), &unit),
        }
      }
    });

    Ok(())
  }

  fn analyze_single_file(&self, path: &String, unit: &Option<FileUnit>) -> Result<()> {
    let f = Path::new(&path);
    let f_name = f.file_name().unwrap().to_os_string().into_string().unwrap();
    let f_meta = f.metadata().expect("cannot extract metadata");

    match self.compression {
      FileCompression::Brotli => {
        let br_size = self.get_brotli_size(&path)?;
        self.analyze_file_size(f_name, br_size.try_into().unwrap(), &unit);
        Ok(())
      }
      FileCompression::UnCompressed => {
        self.analyze_file_size(f_name, f_meta.size(), &unit);
        Ok(())
      }
    }
  }

  pub fn analyze(&self, path: String, unit: Option<FileUnit>) -> Result<()> {
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

  fn analyze_file_size(&self, f_name: String, f_size: u64, u: &Option<FileUnit>) {
    match u {
      Some(v) => {
        let conv_unit = v.get_converted_unit(f_size);
        let budget_size = v.get_budget();
        let is_pass = self.is_budget_pass(conv_unit, budget_size);
        self.f_size_map.lock().unwrap().insert(
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
