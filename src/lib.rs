#![deny(clippy::all)]

use fxhash::FxHashMap;
use std::{os::unix::prelude::MetadataExt, path::Path};

mod config;

#[macro_use]
extern crate napi_derive;

struct BundleChecker {
  file_map: FxHashMap<String, u8>,
}

#[napi(object)]
pub struct CheckBundlerInput {
  pub config_path: String,
}

fn bytes_to_kilobytes(bytes: u64) -> f64 {
  (bytes as f64) / 1024.0
}

#[napi]
pub fn check_bundler(input: CheckBundlerInput) {
  let _bundle_checker = BundleChecker {
    file_map: Default::default(),
  };

  let config = config::get_config(&input.config_path);

  config.bundlesize.iter().for_each(|c| {
    let path = &c.path;
    let is_glob_path = is_glob::is_glob(path);

    println!("analyze from path={}", path);

    if is_glob_path {
      for f in globwalk::glob(path).expect("dir cannot be walked") {
        let dir_entry = f.expect("readable dir entry");
        let meta = dir_entry.metadata().expect("cannot extract metadata");
        if meta.is_file() {
          let file_size_in_kb = bytes_to_kilobytes(meta.size());
          println!(
            "file={}, size={}, max_size={}",
            dir_entry.file_name().to_string_lossy(),
            file_size_in_kb,
            c.max_size,
          )
        }
      }
    } else {
      let f = Path::new(path);
      let f_meta = f.metadata().expect("cannot extract metadata");
      let file_size_in_kb = bytes_to_kilobytes(f_meta.size());
      println!(
        "file={}, size={}, max_size={}",
        f.file_name().unwrap().to_string_lossy(),
        file_size_in_kb,
        c.max_size,
      )
    }
  });
}
