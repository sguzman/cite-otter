use std::fs;
use std::path::Path;

use cite_otter::cli::{
  delta_report,
  training_report,
  validation_report
};
use serde_json::Value;

fn report_path(
  name: &str
) -> std::path::PathBuf {
  Path::new("target")
    .join("reports")
    .join(name)
}

#[test]
fn training_validation_delta_flow_runs()
{
  let reports_dir =
    Path::new("target").join("reports");
  let _ =
    fs::remove_dir_all(&reports_dir);

  training_report()
    .expect("training should succeed");
  validation_report().expect(
    "validation should succeed"
  );
  delta_report()
    .expect("delta should succeed");

  let training = fs::read_to_string(
    report_path("training-report.json")
  )
  .expect(
    "training report should exist"
  );
  let validation =
    fs::read_to_string(report_path(
      "validation-report.json"
    ))
    .expect(
      "validation report should exist"
    );
  let delta = fs::read_to_string(
    report_path("delta-report.json")
  )
  .expect("delta report should exist");

  let training_json: Value =
    serde_json::from_str(&training)
      .expect(
        "training report should parse"
      );
  assert!(
    training_json
      .get("parser")
      .and_then(Value::as_array)
      .map(|arr| !arr.is_empty())
      .unwrap_or(false),
    "training report should list \
     parser datasets"
  );

  let validation_json: Value =
    serde_json::from_str(&validation)
      .expect(
        "validation report should \
         parse"
      );
  assert!(
    validation_json
      .get("parser")
      .and_then(Value::as_array)
      .map(|arr| !arr.is_empty())
      .unwrap_or(false),
    "validation report should list \
     parser datasets"
  );

  let delta_json: Value =
    serde_json::from_str(&delta)
      .expect(
        "delta report should parse"
      );
  assert!(
    delta_json
      .get("comparisons")
      .and_then(Value::as_array)
      .map(|arr| !arr.is_empty())
      .unwrap_or(false),
    "delta report should list \
     comparisons"
  );
}
