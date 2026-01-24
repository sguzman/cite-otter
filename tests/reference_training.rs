use std::fs;
use std::path::{
  Path,
  PathBuf
};

use cite_otter::cli::{
  delta_report,
  training_report,
  validation_report
};
use cite_otter::parser::{
  Parser,
  sequence_signature
};
use cite_otter::sequence_model::SequenceModel;
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
  assert_report_keys(
    &training_json,
    &["parser", "finder", "samples"],
    "training report"
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
  assert_report_keys(
    &validation_json,
    &["parser", "finder"],
    "validation report"
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
  assert_report_keys(
    &delta_json,
    &["comparisons"],
    "delta report"
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

  let finder_dataset_path = Path::new(
    env!("CARGO_MANIFEST_DIR")
  )
  .join(
    "tmp/anystyle/res/finder/\
     bb132pr2055.ttx"
  );
  let canonical_finder_path =
    finder_dataset_path
      .canonicalize()
      .expect(
        "reference finder dataset \
         must exist"
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

  let finder_content =
    fs::read_to_string(
      &finder_dataset_path
    )
    .expect(
      "finder dataset should be \
       readable"
    );
  let expected_finder_sequences =
    parser.label(&finder_content).len();

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
  assert_report_keys(
    training_entry,
    &["path", "sequences", "tokens"],
    "training parser entry"
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

  assert!(
    training_json
      .get("finder")
      .and_then(Value::as_array)
      .map(|arr| !arr.is_empty())
      .unwrap_or(false),
    "training report should list \
     finder datasets"
  );
  let training_finder = training_json
    .get("finder")
    .and_then(Value::as_array)
    .expect(
      "training finder data should be \
       present"
    );
  let finder_entry =
    find_dataset_entry(
      training_finder,
      canonical_finder_path.as_path()
    )
    .expect(
      "training report should include \
       the finder dataset"
    );
  assert_report_keys(
    finder_entry,
    &["path", "sequences", "tokens"],
    "training finder entry"
  );
  let recorded_finder_sequences =
    finder_entry
      .get("sequences")
      .and_then(Value::as_u64)
      .expect(
        "finder sequences must be \
         numeric"
      ) as usize;
  assert_eq!(
    recorded_finder_sequences,
    expected_finder_sequences,
    "finder training stats should \
     reflect parser labels"
  );

  let samples = training_json
    .get("samples")
    .and_then(Value::as_array)
    .expect(
      "training report should list \
       sample outputs"
    );
  assert_eq!(
    samples.len(),
    3,
    "training report should capture \
     each sample format"
  );
  let sample_formats: Vec<_> = samples
    .iter()
    .filter_map(|entry| {
      entry
        .get("format")
        .and_then(Value::as_str)
    })
    .collect();
  for expected in
    ["json", "bibtex", "csl"]
  {
    assert!(
      sample_formats
        .contains(&expected),
      "sample outputs should cover \
       {expected}"
    );
  }
  for entry in samples {
    assert!(
      entry
        .get("output")
        .and_then(Value::as_str)
        .map(|value| !value.is_empty())
        .unwrap_or(false),
      "sample outputs should be \
       non-empty"
    );
    assert_report_keys(
      entry,
      &["format", "output"],
      "sample entry"
    );
  }

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
  assert_report_keys(
    validation_entry,
    &["path", "sequences", "tokens"],
    "validation parser entry"
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

  assert!(
    validation_json
      .get("finder")
      .and_then(Value::as_array)
      .map(|arr| !arr.is_empty())
      .unwrap_or(false),
    "validation report should list \
     finder datasets"
  );
  let validation_finder =
    validation_json
      .get("finder")
      .and_then(Value::as_array)
      .expect(
        "validation finder data \
         should be present"
      );
  let validation_finder_entry =
    find_dataset_entry(
      validation_finder,
      canonical_finder_path.as_path()
    )
    .expect(
      "validation report should cover \
       the finder dataset"
    );
  assert_report_keys(
    validation_finder_entry,
    &["path", "sequences", "tokens"],
    "validation finder entry"
  );
  let validation_finder_sequences =
    validation_finder_entry
      .get("sequences")
      .and_then(Value::as_u64)
      .expect(
        "finder sequences must be \
         numeric"
      ) as usize;
  assert_eq!(
    validation_finder_sequences,
    expected_finder_sequences,
    "finder validation stats should \
     match the training numbers"
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
  assert_report_keys(
    delta_entry,
    &[
      "path", "kind", "prepared",
      "labeled", "stored", "delta"
    ],
    "delta parser entry"
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
  let delta_labeled = delta_entry
    .get("labeled")
    .and_then(Value::as_u64)
    .expect("labeled should be numeric")
    as usize;
  let delta_kind = delta_entry
    .get("kind")
    .and_then(Value::as_str)
    .expect("kind should be present");

  assert_eq!(
    delta_prepared, expected_sequences,
    "delta report should match parser \
     prep counts"
  );
  assert_eq!(
    delta_labeled, expected_sequences,
    "delta report should track parser \
     labels"
  );
  assert_eq!(
    delta_kind, "parser",
    "delta report should tag parser \
     entries"
  );
  assert_eq!(
    delta_stored, expected_sequences,
    "delta report should read the \
     trained model counts"
  );

  let delta_finder_entry =
    find_dataset_entry(
      delta_comparisons,
      canonical_finder_path.as_path()
    )
    .expect(
      "delta report should include \
       the finder dataset"
    );
  assert_report_keys(
    delta_finder_entry,
    &[
      "path", "kind", "prepared",
      "labeled", "stored", "delta"
    ],
    "delta finder entry"
  );
  let delta_finder_prepared =
    delta_finder_entry
      .get("prepared")
      .and_then(Value::as_u64)
      .expect(
        "finder prepared should be \
         numeric"
      ) as usize;
  let delta_finder_labeled =
    delta_finder_entry
      .get("labeled")
      .and_then(Value::as_u64)
      .expect(
        "finder labeled should be \
         numeric"
      ) as usize;
  let delta_finder_kind =
    delta_finder_entry
      .get("kind")
      .and_then(Value::as_str)
      .expect(
        "finder kind should be present"
      );
  assert_eq!(
    delta_finder_prepared,
    expected_finder_sequences,
    "delta report should match finder \
     prep counts"
  );
  assert_eq!(
    delta_finder_labeled,
    expected_finder_sequences,
    "delta report should match finder \
     label counts"
  );
  assert_eq!(
    delta_finder_kind, "finder",
    "delta report should tag finder \
     entries"
  );

  let parser_model_path =
    model_file("parser-sequences.json");
  let parser_model =
    SequenceModel::load(
      &parser_model_path
    )
    .expect(
      "parser sequence model loads"
    );
  assert!(
    parser_model.total() > 0,
    "parser sequence model should \
     record sequences"
  );
  let prepared_dataset =
    parser.prepare(&content, true);
  let first_sequence =
    prepared_dataset.0.first().expect(
      "parser.prepare should yield \
       sequences"
    );
  let signature =
    sequence_signature(first_sequence);
  assert!(
    parser_model.count(&signature) > 0,
    "parser sequence model should \
     keep the signature used in \
     training"
  );

  let finder_model_path =
    model_file("finder-sequences.json");
  let finder_model =
    SequenceModel::load(
      &finder_model_path
    )
    .expect(
      "finder sequence model loads"
    );
  assert!(
    finder_model.total() > 0,
    "finder sequence model should \
     store sequences"
  );
  let finder_json: Value =
    serde_json::from_str(
      &fs::read_to_string(
        &finder_model_path
      )
      .expect(
        "finder sequence model should \
         exist"
      )
    )
    .expect(
      "finder sequence json parses"
    );
  assert!(
    finder_json
      .get("counts")
      .and_then(Value::as_object)
      .map(|counts| !counts.is_empty())
      .unwrap_or(false),
    "finder sequence model should \
     contain signatures"
  );
}

#[test]
fn validation_fails_without_models() {
  let model_dir =
    Path::new("target").join("models");
  let _ =
    fs::remove_dir_all(&model_dir);

  let model_path =
    model_dir.join("parser-model.json");
  assert!(
    !model_path.exists(),
    "parser model should be absent"
  );

  let result = validation_report();
  assert!(
    result.is_ok(),
    "validation should succeed \
     without models"
  );
}

#[test]
fn delta_fails_without_models() {
  let model_dir =
    Path::new("target").join("models");
  let _ =
    fs::remove_dir_all(&model_dir);

  let model_path =
    model_dir.join("parser-model.json");
  assert!(
    !model_path.exists(),
    "parser model should be absent"
  );

  let result = delta_report();
  assert!(
    result.is_ok(),
    "delta should succeed without \
     models"
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

fn assert_report_keys(
  value: &Value,
  expected: &[&str],
  label: &str
) {
  let obj = value
    .as_object()
    .unwrap_or_else(|| {
      panic!(
        "{label} should be a JSON \
         object"
      )
    });
  for key in expected {
    assert!(
      obj.contains_key(*key),
      "{label} should include {key}"
    );
  }
}

fn model_file(name: &str) -> PathBuf {
  Path::new("target")
    .join("models")
    .join(name)
}
