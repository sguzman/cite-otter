use std::fs;
use std::path::Path;

use cite_otter::cli::{
  delta_report,
  training_report,
  validation_report
};
use cite_otter::parser::Parser;
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

  let dataset_path = Path::new(env!(
    "CARGO_MANIFEST_DIR"
  ))
  .join(
    "tmp/anystyle/res/parser/core.xml"
  );
  let canonical_path =
    dataset_path.canonicalize().expect(
      "reference dataset must exist"
    );

  let parser = Parser::new();
  let content =
    fs::read_to_string(&dataset_path)
      .expect(
        "reference dataset should be \
         readable"
      );
  let expected_sequences = parser
    .prepare(&content, true)
    .0
    .len();

  let training_parser = training_json
    .get("parser")
    .and_then(Value::as_array)
    .expect(
      "training parser data should be \
       present"
    );
  let training_entry =
    find_dataset_entry(
      training_parser,
      canonical_path.as_path()
    )
    .expect(
      "training report should include \
       the core parser dataset"
    );
  let recorded_sequences =
    training_entry
      .get("sequences")
      .and_then(Value::as_u64)
      .expect(
        "sequences must be numeric"
      ) as usize;

  assert_eq!(
    recorded_sequences,
    expected_sequences,
    "parser sequencing stats should \
     match the training data"
  );

  let validation_parser =
    validation_json
      .get("parser")
      .and_then(Value::as_array)
      .expect(
        "validation parser data \
         should be present"
      );
  let validation_entry =
    find_dataset_entry(
      validation_parser,
      canonical_path.as_path()
    )
    .expect(
      "validation report should cover \
       the core parser dataset"
    );
  let validation_sequences =
    validation_entry
      .get("sequences")
      .and_then(Value::as_u64)
      .expect(
        "sequences must be numeric"
      ) as usize;

  assert_eq!(
    validation_sequences,
    expected_sequences,
    "validation stats should follow \
     the training numbers"
  );

  let delta_comparisons = delta_json
    .get("comparisons")
    .and_then(Value::as_array)
    .expect(
      "delta comparisons should be \
       present"
    );
  let delta_entry = find_dataset_entry(
    delta_comparisons,
    canonical_path.as_path()
  )
  .expect(
    "delta report should include the \
     core parser dataset"
  );
  let delta_prepared = delta_entry
    .get("prepared")
    .and_then(Value::as_u64)
    .expect(
      "prepared should be numeric"
    ) as usize;
  let delta_stored = delta_entry
    .get("stored")
    .and_then(Value::as_u64)
    .expect("stored should be numeric")
    as usize;

  assert_eq!(
    delta_prepared, expected_sequences,
    "delta report should match parser \
     prep counts"
  );
  assert_eq!(
    delta_stored, expected_sequences,
    "delta report should read the \
     trained model counts"
  );
}

fn find_dataset_entry<'a>(
  entries: &'a [Value],
  target: &Path
) -> Option<&'a Value> {
  entries.iter().find(|entry| {
    entry
      .get("path")
      .and_then(Value::as_str)
      .and_then(|path| {
        Path::new(path)
          .canonicalize()
          .ok()
      })
      .map(|canonical| {
        canonical == target
      })
      .unwrap_or(false)
  })
}
