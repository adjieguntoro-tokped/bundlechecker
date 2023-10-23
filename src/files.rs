use std::{
  io::Read,
  os::unix::prelude::MetadataExt,
  path::Path,
  sync::{Arc, Mutex},
};

use crate::config::BundleConfig;
use anyhow::{anyhow, Result};
use rayon::prelude::*;

pub enum FileUnit {
  Kb(f64),
}

impl FileUnit {
  fn bytes_to_kilobytes(&self, size: u64) -> f64 {
    (size as f64) / 1024.0
  }

  pub fn get_converted_unit(&self, file_size: u64) -> f64 {
    match self {
      Self::Kb(_) => self.bytes_to_kilobytes(file_size),
    }
  }

  pub fn to_string(&self) -> String {
    match self {
      Self::Kb(_) => "kB".to_string(),
    }
  }
}

fn get_file_unit(size: &str) -> Option<FileUnit> {
  let split_size: Vec<&str> = size.split(" ").collect();
  if split_size.len() == 2 {
    let num_size = split_size[0];
    let num_unit = split_size[1];
    match num_unit {
      "kB" => {
        let i = num_size.parse::<f64>().expect("cannot parse size");
        Some(FileUnit::Kb(i))
      }
      _ => None,
    }
  } else {
    None
  }
}

#[derive(Debug)]
pub enum FileCompression {
  Brotli,
  UnCompressed,
}

impl FileCompression {
  pub fn to_string(&self) -> String {
    match self {
      FileCompression::UnCompressed => "UnCompressed".to_string(),
      FileCompression::Brotli => "Brotli".to_string(),
    }
  }
}

pub fn get_file_compression(compression: &str) -> FileCompression {
  match compression {
    "brotli" => FileCompression::Brotli,
    _ => FileCompression::UnCompressed,
  }
}

#[derive(Debug, Clone)]
pub struct File {
  pub budget_size: f64,
  pub actual_file_size: f64,
  pub compression: String,
  pub size_unit: String,
  pub error: Option<String>,
}

pub struct Files {
  bundlesize_config: Vec<BundleConfig>,
  brotli_enc_params: brotli::enc::BrotliEncoderParams,
  compression: FileCompression,
}

impl Files {
  pub fn new(conf: Vec<BundleConfig>, compression: FileCompression) -> Self {
    Files {
      bundlesize_config: conf,
      brotli_enc_params: brotli::enc::BrotliEncoderParams::default(),
      compression,
    }
  }

  fn convert_max_budget_unit(&self, max_budget: &String) -> Result<f64> {
    let file_unit = get_file_unit(&max_budget);
    if let Some(v) = file_unit {
      match v {
        FileUnit::Kb(v) => Ok(v),
      }
    } else {
      Err(anyhow!("unit not supported"))
    }
  }

  fn convert_actual_size_unit(&self, f_size: u64, unit: &Option<FileUnit>) -> Result<f64> {
    match unit {
      Some(v) => {
        let conv_unit = v.get_converted_unit(f_size);
        Ok(conv_unit)
      }
      _ => Err(anyhow!("unit not supported")),
    }
  }

  fn get_brotli_size(&self, file_path: &str) -> Result<f64> {
    let f = std::fs::File::open(&file_path);
    match f {
      Ok(file) => {
        let mut buf = vec![];
        let mut writer = brotli::CompressorReader::with_params(file, 4096, &self.brotli_enc_params);
        let b_size = writer.read_to_end(&mut buf)?;
        Ok(b_size as f64)
      }
      Err(_) => Err(anyhow!("{file_path} is not found")),
    }
  }

  fn collect_glob(
    &self,
    c: &BundleConfig,
    collected_files: &Arc<Mutex<fxhash::FxHashMap<String, File>>>,
  ) -> Result<()> {
    let path = &c.path;
    let budget_size = self.convert_max_budget_unit(&c.max_size)?;

    let mut glob_walker = globwalk::glob(path)?.peekable();
    if glob_walker.peek().is_none() {
      collected_files.lock().unwrap().insert(
        path.to_string(),
        File {
          budget_size,
          actual_file_size: 0.0,
          size_unit: "".to_string(),
          compression: "".to_string(),
          error: Some(format!("pattern {} is not getting any match", path)),
        },
      );
    }

    let mut v: Vec<globwalk::DirEntry> = vec![];
    for f in glob_walker {
      let dir_entry = f?;
      v.push(dir_entry);
    }

    v.par_iter().try_for_each(|dir_entry| -> Result<()> {
      let f_meta = dir_entry.metadata()?;
      let file_unit = get_file_unit(&c.max_size);

      if f_meta.is_file() {
        let f_name = dir_entry.file_name().to_os_string().into_string().unwrap();
        match self.compression {
          FileCompression::Brotli => {
            if let Some(p) = dir_entry.path().to_str() {
              let actual_file_size = self.get_brotli_size(p)?;
              collected_files.lock().unwrap().insert(
                f_name,
                File {
                  budget_size,
                  actual_file_size,
                  size_unit: file_unit.unwrap().to_string(),
                  compression: self.compression.to_string(),
                  error: None,
                },
              );
              Ok(())
            } else {
              Err(anyhow!("unexpected condition"))
            }
          }
          FileCompression::UnCompressed => {
            let actual_file_size = self.convert_actual_size_unit(f_meta.size(), &file_unit)?;
            collected_files.lock().unwrap().insert(
              f_name,
              File {
                budget_size,
                actual_file_size,
                size_unit: file_unit.unwrap().to_string(),
                compression: self.compression.to_string(),
                error: None,
              },
            );
            Ok(())
          }
        }
      } else {
        Ok(())
      }
    })?;

    Ok(())
  }

  fn collect_single_file(
    &self,
    c: &BundleConfig,
    collected_files: &Arc<Mutex<fxhash::FxHashMap<String, File>>>,
  ) -> Result<()> {
    let path = &c.path;
    let file_unit = get_file_unit(&c.max_size);

    let f = Path::new(path);
    let budget_size = self.convert_max_budget_unit(&c.max_size)?;

    let file_name = f.file_name();
    if let Some(f_name) = file_name {
      let f_meta = f.metadata();

      if let Err(_) = f_meta {
        return Err(anyhow!("single file for {path} does not exist"));
      }

      match self.compression {
        FileCompression::Brotli => {
          let actual_file_size = self.get_brotli_size(path)?;
          collected_files.lock().unwrap().insert(
            f_name.to_string_lossy().to_string(),
            File {
              budget_size,
              actual_file_size,
              size_unit: file_unit.unwrap().to_string(),
              compression: self.compression.to_string(),
              error: None,
            },
          );
          return Ok(());
        }
        FileCompression::UnCompressed => {
          let actual_file_size = self.convert_actual_size_unit(f_meta?.size(), &file_unit)?;
          collected_files.lock().unwrap().insert(
            f_name.to_string_lossy().to_string(),
            File {
              budget_size,
              actual_file_size,
              size_unit: file_unit.unwrap().to_string(),
              compression: self.compression.to_string(),
              error: None,
            },
          );
          return Ok(());
        }
      }
    }

    Err(anyhow!("{path} cannot be converted to path"))
  }

  pub fn collect(&self) -> Result<fxhash::FxHashMap<String, File>> {
    let collected = Arc::new(Mutex::new(fxhash::FxHashMap::default()));
    self
      .bundlesize_config
      .par_iter()
      .try_for_each(|f| -> Result<()> {
        let path = &f.path;
        let is_glob_file = is_glob::is_glob(path);
        if is_glob_file {
          self.collect_glob(&f, &collected)?;
          Ok(())
        } else {
          self.collect_single_file(&f, &collected)?;
          Ok(())
        }
      })?;

    let collected_files = collected.lock().unwrap();
    Ok(collected_files.to_owned())
  }
}
