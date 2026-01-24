use std::fs;
use std::process::Command;

#[test]
fn ruby_format_parity_matches() {
  let enabled = std::env::var(
    "CITE_OTTER_RUBY_PARITY"
  )
  .ok()
  .map(|value| value == "1")
  .unwrap_or(false);
  if !enabled {
    return;
  }

  let status = Command::new("bash")
    .arg("scripts/compare_ruby_format.sh")
    .status()
    .expect("run ruby format parity");
  assert!(
    status.success(),
    "ruby format parity script failed"
  );

  let report = fs::read_to_string(
    "target/reports/ruby-format-diff.txt"
  )
  .expect("read ruby format diff");
  assert!(
    !report.contains("\n--- ")
      && !report.contains("\n+++ "),
    "ruby format diff report contains changes"
  );
}
