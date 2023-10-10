use std::fs;

pub fn slurp_file(file_name: &str) -> Vec<u8> {
  fs::read(file_name).unwrap_or_else(|_| panic!("Unable to read file {file_name}"))
}

#[cfg(test)]
mod tests {
  use std::{fs::File, io::Write};

  use super::*;

  #[test]
  fn test_slurp_file() {
    let file_name = "test.txt";
    let content = b"Hello, world!";

    let mut file = File::create(file_name).unwrap();
    file.write_all(content).unwrap();

    let result = slurp_file(file_name);

    assert_eq!(result, content);

    std::fs::remove_file(file_name).unwrap();
  }

  #[test]
  #[should_panic(expected = "Unable to read file")]
  fn test_slurp_file_nonexistent() {
    let file_name = "nonexistent.txt";

    slurp_file(file_name);
  }
}
