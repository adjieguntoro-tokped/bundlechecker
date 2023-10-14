pub enum FileUnit {
  Kb(f64),
}

impl FileUnit {
  fn bytes_to_kilobytes(&self, size: u64) -> f64 {
    (size as f64) / 1024.0
  }

  pub fn get_budget(&self) -> f64 {
    match self {
      Self::Kb(v) => *v,
    }
  }

  pub fn get_converted_unit(&self, file_size: u64) -> f64 {
    match self {
      Self::Kb(_) => self.bytes_to_kilobytes(file_size),
    }
  }
}

pub fn get_file_unit(size: &str) -> Option<FileUnit> {
  let split_size: Vec<&str> = size.split(" ").collect();
  if split_size.len() == 2 {
    let num_size = split_size[0];
    let num_unit = split_size[1];
    match num_unit {
      "kB" => {
        let i = num_size.parse::<f64>().expect("cannot parse size");
        Some(FileUnit::Kb(i))
      }
      "" => None,
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

pub fn get_file_compression(compression: &str) -> FileCompression {
  match compression {
    "brotli" => FileCompression::Brotli,
    _ => FileCompression::UnCompressed,
  }
}
