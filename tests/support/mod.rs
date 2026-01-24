use std::path::{
  Path,
  PathBuf
};

#[allow(dead_code)]
pub fn fixture_path(
  path: &str
) -> PathBuf {
  Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("tests/fixtures")
    .join(path)
}

#[allow(dead_code)]
pub fn assert_snapshot_eq(
  label: &str,
  actual: &str,
  expected: &str
) {
  if actual.trim_end() == expected.trim_end() {
    return;
  }

  eprintln!(
    "\nsnapshot mismatch: {label}\n{}",
    diff_lines(expected, actual)
  );
  panic!("snapshot mismatch: {label}");
}

#[allow(dead_code)]
fn diff_lines(
  expected: &str,
  actual: &str
) -> String {
  let expected_lines =
    expected.lines().collect::<Vec<_>>();
  let actual_lines =
    actual.lines().collect::<Vec<_>>();
  let mut out = Vec::new();
  out.push("--- expected".to_string());
  out.push("+++ actual".to_string());

  let max = expected_lines
    .len()
    .max(actual_lines.len());
  for idx in 0..max {
    let left = expected_lines
      .get(idx)
      .copied()
      .unwrap_or("");
    let right = actual_lines
      .get(idx)
      .copied()
      .unwrap_or("");
    if left == right {
      continue;
    }
    out.push(format!("-{left}"));
    out.push(format!("+{right}"));
  }
  out.join("\n")
}
