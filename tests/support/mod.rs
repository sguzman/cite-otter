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

  let (diff, summary) =
    diff_lines(expected, actual);
  let header =
    snapshot_header(label, &summary);
  let report = format!("{header}\n{diff}");
  let report_path = Path::new("target")
    .join("reports")
    .join("format-diff.txt");
  if let Some(parent) = report_path.parent() {
    let _ = std::fs::create_dir_all(parent);
  }
  let _ = std::fs::write(&report_path, &report);
  eprintln!(
    "\nsnapshot mismatch: {label}\n{diff}\n(diff saved to {})",
    report_path.display()
  );
  panic!("snapshot mismatch: {label}");
}

#[allow(dead_code)]
fn diff_lines(
  expected: &str,
  actual: &str
) -> (String, DiffSummary) {
  let expected_lines =
    expected.lines().collect::<Vec<_>>();
  let actual_lines =
    actual.lines().collect::<Vec<_>>();
  let mut out = Vec::new();
  let mut removed = 0usize;
  let mut added = 0usize;
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
    if !left.is_empty() {
      out.push(format!("-{left}"));
      removed += 1;
    }
    if !right.is_empty() {
      out.push(format!("+{right}"));
      added += 1;
    }
  }
  (
    out.join("\n"),
    DiffSummary {
      expected_lines: expected_lines.len(),
      actual_lines: actual_lines.len(),
      removed,
      added
    }
  )
}

struct DiffSummary {
  expected_lines: usize,
  actual_lines: usize,
  removed: usize,
  added: usize
}

fn snapshot_header(
  label: &str,
  summary: &DiffSummary
) -> String {
  let timestamp = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map(|duration| duration.as_secs())
    .unwrap_or(0);
  format!(
    "snapshot: {label}\nupdated: {timestamp}\nexpected_lines: {}\nactual_lines: {}\nremoved: {}\nadded: {}",
    summary.expected_lines,
    summary.actual_lines,
    summary.removed,
    summary.added
  )
}
