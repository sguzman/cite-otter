use std::fs;
use std::path::{
  Path,
  PathBuf
};

use cite_otter::cli::{
  delta_report_with_paths,
  training_report_with_paths,
  validation_report_with_paths
};
use cite_otter::parser::{
  Parser,
  sequence_signature
};
use cite_otter::sequence_model::SequenceModel;
use serde_json::Value;
use tempfile::tempdir;

fn report_path(
  report_dir: &Path,
  name: &str
) -> std::path::PathBuf {
  report_dir.join(name)
}

fn isolated_dirs()
-> (tempfile::TempDir, PathBuf, PathBuf)
{
  let temp =
    tempdir().expect("temp dir");
  let model_dir =
    temp.path().join("models");
  let report_dir =
    temp.path().join("reports");
  (temp, model_dir, report_dir)
}

#[test]
fn training_validation_delta_flow_runs()
{
  let (_temp, model_dir, report_dir) =
    isolated_dirs();

  training_report_with_paths(
    &model_dir,
    &report_dir
  )
  .expect("training should succeed");
  validation_report_with_paths(
    &model_dir,
    &report_dir
  )
  .expect("validation should succeed");
  delta_report_with_paths(
    &model_dir,
    &report_dir
  )
  .expect("delta should succeed");

  let training =
    fs::read_to_string(report_path(
      &report_dir,
      "training-report.json"
    ))
    .expect(
      "training report should exist"
    );
  let validation =
    fs::read_to_string(report_path(
      &report_dir,
      "validation-report.json"
    ))
    .expect(
      "validation report should exist"
    );
  let delta = fs::read_to_string(
    report_path(
      &report_dir,
      "delta-report.json"
    )
  )
  .expect("delta report should exist");

  let training_json: Value =
    serde_json::from_str(&training)
      .expect(
        "training report should parse"
      );
  assert_report_keys(
    &training_json,
    &[
      "parser", "finder", "samples",
      "summary"
    ],
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
  let samples = training_json
    .get("samples")
    .and_then(Value::as_array)
    .expect("training samples");
  let mut formats = samples
    .iter()
    .filter_map(|entry| {
      entry
        .get("format")
        .and_then(Value::as_str)
    })
    .collect::<Vec<_>>();
  formats.sort();
  formats.dedup();
  assert_eq!(
    formats,
    vec!["bibtex", "csl", "json"],
    "training samples should include \
     all output formats"
  );
  for sample in samples {
    let references = sample
      .get("references")
      .and_then(Value::as_u64)
      .expect("sample references");
    assert!(
      references > 0,
      "sample references should be \
       non-zero"
    );
    let output = sample
      .get("output")
      .and_then(Value::as_str)
      .expect("sample output");
    assert!(
      !output.trim().is_empty(),
      "sample output should not be \
       empty"
    );
  }

  let validation_json: Value =
    serde_json::from_str(&validation)
      .expect(
        "validation report should \
         parse"
      );
  assert_report_keys(
    &validation_json,
    &["parser", "finder", "summary"],
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
  let validation_summary =
    validation_json
      .get("summary")
      .expect("validation summary");
  assert_report_keys(
    validation_summary,
    &["parser", "finder"],
    "validation summary"
  );
  for key in ["parser", "finder"] {
    let summary = validation_summary
      .get(key)
      .expect("summary entry");
    assert_report_keys(
      summary,
      &[
        "datasets",
        "sequences",
        "tokens"
      ],
      "validation summary entry"
    );
  }

  let delta_json: Value =
    serde_json::from_str(&delta)
      .expect(
        "delta report should parse"
      );
  assert_report_keys(
    &delta_json,
    &["comparisons", "summary"],
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
  let comparisons = delta_json
    .get("comparisons")
    .and_then(Value::as_array)
    .expect("comparisons");
  let first = comparisons
    .first()
    .expect("comparison entry");
  assert_report_keys(
    first,
    &[
      "path",
      "kind",
      "prepared",
      "labeled",
      "stored",
      "delta",
      "prepared_tokens",
      "labeled_tokens",
      "stored_tokens",
      "delta_tokens"
    ],
    "delta comparison entry"
  );
  let delta_summary = delta_json
    .get("summary")
    .expect("delta summary");
  assert_report_keys(
    delta_summary,
    &[
      "comparisons",
      "parser",
      "finder"
    ],
    "delta summary"
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
  let expected_finder_tokens: usize =
    parser
      .label(&finder_content)
      .iter()
      .map(|sequence| sequence.len())
      .sum();

  let training_parser = training_json
    .get("parser")
    .and_then(Value::as_array)
    .expect(
      "training parser data should be \
       present"
    );
  let training_finder = training_json
    .get("finder")
    .and_then(Value::as_array)
    .expect(
      "training finder data should be \
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
  let recorded_tokens = training_entry
    .get("tokens")
    .and_then(Value::as_u64)
    .expect("tokens must be numeric")
    as usize;

  assert_eq!(
    recorded_sequences,
    expected_sequences,
    "parser sequencing stats should \
     match the training data"
  );
  let expected_tokens: usize = parser
    .prepare(&content, true)
    .0
    .iter()
    .map(|sequence| sequence.len())
    .sum();
  assert_eq!(
    recorded_tokens, expected_tokens,
    "parser token counts should match \
     training data"
  );
  let summary = training_json
    .get("summary")
    .expect("training summary");
  assert_report_keys(
    summary,
    &["parser", "finder", "samples"],
    "training summary"
  );
  assert_eq!(
    summary
      .get("parser")
      .and_then(|value| {
        value.get("datasets")
      })
      .and_then(Value::as_u64),
    Some(training_parser.len() as u64),
    "training summary should count \
     parser datasets"
  );
  assert_eq!(
    summary
      .get("parser")
      .and_then(|value| {
        value.get("sequences")
      })
      .and_then(Value::as_u64),
    Some(
      training_parser
        .iter()
        .filter_map(|entry| {
          entry
            .get("sequences")
            .and_then(Value::as_u64)
        })
        .sum()
    ),
    "training summary should track \
     parser sequences"
  );
  assert_eq!(
    summary
      .get("parser")
      .and_then(|value| {
        value.get("tokens")
      })
      .and_then(Value::as_u64),
    Some(
      training_parser
        .iter()
        .filter_map(|entry| {
          entry
            .get("tokens")
            .and_then(Value::as_u64)
        })
        .sum()
    ),
    "training summary should track \
     parser tokens"
  );
  assert_eq!(
    summary
      .get("finder")
      .and_then(|value| {
        value.get("datasets")
      })
      .and_then(Value::as_u64),
    Some(training_finder.len() as u64),
    "training summary should count \
     finder datasets"
  );
  assert_eq!(
    summary
      .get("finder")
      .and_then(|value| {
        value.get("sequences")
      })
      .and_then(Value::as_u64),
    Some(
      training_finder
        .iter()
        .filter_map(|entry| {
          entry
            .get("sequences")
            .and_then(Value::as_u64)
        })
        .sum()
    ),
    "training summary should track \
     finder sequences"
  );
  assert_eq!(
    summary
      .get("finder")
      .and_then(|value| {
        value.get("tokens")
      })
      .and_then(Value::as_u64),
    Some(
      training_finder
        .iter()
        .filter_map(|entry| {
          entry
            .get("tokens")
            .and_then(Value::as_u64)
        })
        .sum()
    ),
    "training summary should track \
     finder tokens"
  );
  assert_eq!(
    summary
      .get("samples")
      .and_then(Value::as_u64),
    Some(3),
    "training summary should track \
     samples"
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
  let recorded_finder_tokens =
    finder_entry
      .get("tokens")
      .and_then(Value::as_u64)
      .expect(
        "finder tokens must be numeric"
      ) as usize;
  assert_eq!(
    recorded_finder_sequences,
    expected_finder_sequences,
    "finder training stats should \
     reflect parser labels"
  );
  assert_eq!(
    recorded_finder_tokens,
    expected_finder_tokens,
    "finder token counts should \
     reflect labeled sequences"
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
      &[
        "format",
        "output",
        "references"
      ],
      "sample entry"
    );
    let reference_count = entry
      .get("references")
      .and_then(Value::as_u64)
      .expect(
        "sample entry should include \
         reference count"
      );
    assert!(
      reference_count > 0,
      "sample entry reference counts \
       should be non-zero"
    );
    if let Some(format) = entry
      .get("format")
      .and_then(Value::as_str)
      && let Some(output) = entry
        .get("output")
        .and_then(Value::as_str)
    {
      match format {
        | "json" => {
          let parsed: Value =
            serde_json::from_str(
              output
            )
            .expect(
              "json sample should \
               parse"
            );
          assert!(
            parsed.is_array(),
            "json sample should be an \
             array"
          );
        }
        | "csl" => {
          let first = output
            .lines()
            .find(|line| {
              !line.trim().is_empty()
            })
            .expect(
              "csl sample should have \
               lines"
            );
          let parsed: Value =
            serde_json::from_str(first)
              .expect(
                "csl sample line \
                 should parse"
              );
          assert!(
            parsed.is_object(),
            "csl sample should be \
             JSON objects"
          );
        }
        | "bibtex" => {
          assert!(
            output
              .trim_start()
              .starts_with('@'),
            "bibtex sample should \
             start with @"
          );
        }
        | _ => {}
      }
    }
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
    &[
      "path",
      "sequences",
      "tokens",
      "stored_sequences",
      "stored_tokens",
      "delta_sequences",
      "delta_tokens",
      "delta_rate",
      "token_rate"
    ],
    "validation parser entry"
  );
  let validation_sequences =
    validation_entry
      .get("sequences")
      .and_then(Value::as_u64)
      .expect(
        "sequences must be numeric"
      ) as usize;
  let validation_tokens =
    validation_entry
      .get("tokens")
      .and_then(Value::as_u64)
      .expect("tokens must be numeric")
      as usize;
  let validation_stored_sequences =
    validation_entry
      .get("stored_sequences")
      .and_then(Value::as_u64)
      .expect(
        "stored sequences numeric"
      ) as usize;
  let validation_stored_tokens =
    validation_entry
      .get("stored_tokens")
      .and_then(Value::as_u64)
      .expect("stored tokens numeric")
      as usize;
  let validation_delta_sequences =
    validation_entry
      .get("delta_sequences")
      .and_then(Value::as_u64)
      .expect("delta sequences numeric")
      as usize;
  let validation_delta_tokens =
    validation_entry
      .get("delta_tokens")
      .and_then(Value::as_u64)
      .expect("delta tokens numeric")
      as usize;
  let validation_delta_rate =
    validation_entry
      .get("delta_rate")
      .and_then(Value::as_f64)
      .expect("delta rate numeric");
  let validation_token_rate =
    validation_entry
      .get("token_rate")
      .and_then(Value::as_f64)
      .expect("token rate numeric");

  assert_eq!(
    validation_sequences,
    expected_sequences,
    "validation stats should follow \
     the training numbers"
  );
  assert_eq!(
    validation_tokens, expected_tokens,
    "validation token counts should \
     match training data"
  );
  assert_eq!(
    validation_stored_sequences,
    expected_sequences,
    "validation stored sequences \
     should match training data"
  );
  assert_eq!(
    validation_stored_tokens,
    expected_tokens,
    "validation stored tokens should \
     match training data"
  );
  assert_eq!(
    validation_delta_sequences, 0,
    "validation delta sequences \
     should be zero"
  );
  assert_eq!(
    validation_delta_tokens, 0,
    "validation delta tokens should \
     be zero"
  );
  assert_eq!(
    validation_delta_rate, 0.0,
    "validation delta rate should be \
     zero"
  );
  assert_eq!(
    validation_token_rate, 0.0,
    "validation token rate should be \
     zero"
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
    &[
      "path",
      "sequences",
      "tokens",
      "stored_sequences",
      "stored_tokens",
      "delta_sequences",
      "delta_tokens",
      "delta_rate",
      "token_rate"
    ],
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
  let validation_finder_tokens =
    validation_finder_entry
      .get("tokens")
      .and_then(Value::as_u64)
      .expect(
        "finder tokens must be numeric"
      ) as usize;
  let validation_finder_stored_tokens =
    validation_finder_entry
      .get("stored_tokens")
      .and_then(Value::as_u64)
      .expect(
        "finder stored tokens numeric"
      ) as usize;
  let validation_finder_stored =
    validation_finder_entry
      .get("stored_sequences")
      .and_then(Value::as_u64)
      .expect(
        "finder stored sequences \
         numeric"
      ) as usize;
  let validation_finder_delta =
    validation_finder_entry
      .get("delta_sequences")
      .and_then(Value::as_u64)
      .expect(
        "finder delta sequences \
         numeric"
      ) as usize;
  let validation_finder_token_delta =
    validation_finder_entry
      .get("delta_tokens")
      .and_then(Value::as_u64)
      .expect(
        "finder delta tokens numeric"
      ) as usize;
  let validation_finder_rate =
    validation_finder_entry
      .get("delta_rate")
      .and_then(Value::as_f64)
      .expect(
        "finder delta rate numeric"
      );
  let validation_finder_token_rate =
    validation_finder_entry
      .get("token_rate")
      .and_then(Value::as_f64)
      .expect(
        "finder token rate numeric"
      );
  let validation_summary =
    validation_json
      .get("summary")
      .expect("validation summary");
  assert_report_keys(
    validation_summary,
    &["parser", "finder"],
    "validation summary"
  );
  assert_eq!(
    validation_summary
      .get("parser")
      .and_then(|value| {
        value.get("datasets")
      })
      .and_then(Value::as_u64),
    Some(validation_parser.len() as u64),
    "validation summary should count \
     parser datasets"
  );
  assert_eq!(
    validation_summary
      .get("finder")
      .and_then(|value| {
        value.get("datasets")
      })
      .and_then(Value::as_u64),
    Some(validation_finder.len() as u64),
    "validation summary should count \
     finder datasets"
  );
  assert_eq!(
    validation_finder_sequences,
    expected_finder_sequences,
    "finder validation stats should \
     match the training numbers"
  );
  assert_eq!(
    validation_finder_tokens,
    expected_finder_tokens,
    "finder validation token counts \
     should match labeled sequences"
  );
  assert_eq!(
    validation_finder_stored,
    expected_finder_sequences,
    "finder stored sequences should \
     match training data"
  );
  assert_eq!(
    validation_finder_stored_tokens,
    expected_finder_tokens,
    "finder stored tokens should \
     match training data"
  );
  assert_eq!(
    validation_finder_delta, 0,
    "finder delta sequences should be \
     zero"
  );
  assert_eq!(
    validation_finder_token_delta, 0,
    "finder delta tokens should be \
     zero"
  );
  assert_eq!(
    validation_finder_rate, 0.0,
    "finder delta rate should be zero"
  );
  assert_eq!(
    validation_finder_token_rate, 0.0,
    "finder token rate should be zero"
  );

  let delta_comparisons = delta_json
    .get("comparisons")
    .and_then(Value::as_array)
    .expect(
      "delta comparisons should be \
       present"
    );
  let delta_summary = delta_json
    .get("summary")
    .expect("delta summary");
  assert_report_keys(
    delta_summary,
    &[
      "comparisons",
      "parser",
      "finder"
    ],
    "delta summary"
  );
  assert_eq!(
    delta_summary
      .get("comparisons")
      .and_then(Value::as_u64),
    Some(delta_comparisons.len() as u64),
    "delta summary should count \
     comparisons"
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
      "path",
      "kind",
      "prepared",
      "labeled",
      "stored",
      "delta",
      "prepared_tokens",
      "labeled_tokens",
      "stored_tokens",
      "delta_tokens"
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
  let delta_prepared_tokens =
    delta_entry
      .get("prepared_tokens")
      .and_then(Value::as_u64)
      .expect("prepared tokens numeric")
      as usize;
  let delta_labeled_tokens = delta_entry
    .get("labeled_tokens")
    .and_then(Value::as_u64)
    .expect("labeled tokens numeric")
    as usize;
  let delta_stored_tokens = delta_entry
    .get("stored_tokens")
    .and_then(Value::as_u64)
    .expect("stored tokens numeric")
    as usize;
  let delta_tokens = delta_entry
    .get("delta_tokens")
    .and_then(Value::as_i64)
    .expect("delta tokens numeric");
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
  assert_eq!(
    delta_prepared_tokens,
    expected_tokens,
    "delta report should track \
     prepared tokens"
  );
  assert_eq!(
    delta_labeled_tokens,
    expected_tokens,
    "delta report should track \
     labeled tokens"
  );
  assert_eq!(
    delta_stored_tokens,
    expected_tokens,
    "delta report should track stored \
     tokens"
  );
  assert_eq!(
    delta_tokens, 0,
    "delta report should track zero \
     token delta"
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
      "path",
      "kind",
      "prepared",
      "labeled",
      "stored",
      "delta",
      "prepared_tokens",
      "labeled_tokens",
      "stored_tokens",
      "delta_tokens"
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
  let delta_finder_tokens =
    delta_finder_entry
      .get("labeled_tokens")
      .and_then(Value::as_u64)
      .expect(
        "finder labeled tokens should \
         be numeric"
      ) as usize;
  let delta_finder_stored_tokens =
    delta_finder_entry
      .get("stored_tokens")
      .and_then(Value::as_u64)
      .expect(
        "finder stored tokens should \
         be numeric"
      ) as usize;
  let delta_finder_token_delta =
    delta_finder_entry
      .get("delta_tokens")
      .and_then(Value::as_i64)
      .expect(
        "finder delta tokens should \
         be numeric"
      );
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
    delta_finder_tokens,
    expected_finder_tokens,
    "delta report should include \
     finder token counts"
  );
  assert_eq!(
    delta_finder_stored_tokens,
    expected_finder_tokens,
    "delta report should include \
     finder stored token counts"
  );
  assert_eq!(
    delta_finder_token_delta, 0,
    "delta report should include zero \
     finder token delta"
  );
  assert_eq!(
    delta_finder_kind, "finder",
    "delta report should tag finder \
     entries"
  );

  let parser_model_path = model_file(
    &model_dir,
    "parser-sequences.json"
  );
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

  let finder_model_path = model_file(
    &model_dir,
    "finder-sequences.json"
  );
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
  let (_temp, model_dir, report_dir) =
    isolated_dirs();
  let _ =
    fs::remove_dir_all(&model_dir);

  let model_path =
    model_dir.join("parser-model.json");
  assert!(
    !model_path.exists(),
    "parser model should be absent"
  );

  let result =
    validation_report_with_paths(
      &model_dir,
      &report_dir
    );
  assert!(
    result.is_ok(),
    "validation should succeed \
     without models"
  );
}

#[test]
fn delta_fails_without_models() {
  let (_temp, model_dir, report_dir) =
    isolated_dirs();
  let _ =
    fs::remove_dir_all(&model_dir);

  let model_path =
    model_dir.join("parser-model.json");
  assert!(
    !model_path.exists(),
    "parser model should be absent"
  );

  let result = delta_report_with_paths(
    &model_dir,
    &report_dir
  );
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

fn model_file(
  model_dir: &Path,
  name: &str
) -> PathBuf {
  model_dir.join(name)
}
