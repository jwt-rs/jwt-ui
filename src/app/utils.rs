use std::{fs, path::Path};

pub fn slurp_file(file_name: &str) -> Vec<u8> {
  fs::read(file_name).unwrap_or_else(|_| panic!("Unable to read file {file_name}"))
}

pub fn write_file(path: &Path, content: &[u8]) {
  fs::write(path, content).unwrap_or_else(|_| panic!("Unable to write file {}", path.display()))
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {}
