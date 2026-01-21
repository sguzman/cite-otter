use std::path::{
  Path,
  PathBuf
};

pub fn fixture_path(
  path: &str
) -> PathBuf {
  Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("tests/fixtures")
    .join(path)
}
