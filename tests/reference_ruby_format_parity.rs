use std::fs;
use std::process::Command;

use serde_json::Value;

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
    .arg(
      "scripts/compare_ruby_format.sh"
    )
    .status()
    .expect("run ruby format parity");
  assert!(
    status.success(),
    "ruby format parity script failed"
  );

  let report = fs::read_to_string(
    "target/reports/ruby-format-diff.\
     txt"
  )
  .expect("read ruby format diff");
  let summary = fs::read_to_string(
    "target/reports/\
     ruby-format-summary.json"
  )
  .expect("read ruby format summary");
  let summary_json: Value =
    serde_json::from_str(&summary)
      .expect(
        "parse ruby format summary"
      );
  let all_match = summary_json
    .get("all_match")
    .and_then(Value::as_bool)
    .unwrap_or(false);
  let mismatch_summary = summary_json
    .get("comparisons")
    .and_then(Value::as_array)
    .map(|comparisons| {
      comparisons
        .iter()
        .filter_map(|entry| {
          let matches = entry
            .get("matches")
            .and_then(Value::as_bool)
            .unwrap_or(false);
          if matches {
            return None;
          }
          let name = entry
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
          let added = entry
            .get("added_lines")
            .and_then(Value::as_u64)
            .unwrap_or(0);
          let removed = entry
            .get("removed_lines")
            .and_then(Value::as_u64)
            .unwrap_or(0);
          let hunks = entry
            .get("hunks")
            .and_then(Value::as_u64)
            .unwrap_or(0);
          Some(format!(
            "{name} (+{added} / \
             -{removed}, hunks \
             {hunks})"
          ))
        })
        .collect::<Vec<_>>()
        .join(", ")
    })
    .unwrap_or_default();
  assert!(
    all_match
      && !report.contains("\n--- ")
      && !report.contains("\n+++ "),
    "ruby format diff report contains \
     changes: {mismatch_summary}"
  );
}
