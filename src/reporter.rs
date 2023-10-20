pub enum Reporter {
  StandardOutput,
}

pub fn get_reporter(reporter: &str) -> Reporter {
  match reporter {
    "stdout" => Reporter::StandardOutput,
    _ => Reporter::StandardOutput,
  }
}
