use std::collections::{
  BTreeMap,
  BTreeSet
};

use cite_otter::format::ParseFormat;
use cite_otter::parser::FieldValue;

const PREPARED_LINES: [&str; 2] = [
  "Hello, hello Lu P H He , o, \
   initial none F F F F none first \
   other none weak F",
  "world! world Ll P w wo ! d! lower \
   none T F T T none last other none \
   weak F"
];

const PEREC_REF: &str =
  "Perec, Georges. A Void. London: \
   The Harvill Press, 1995. p.108.";

#[test]
#[ignore = "pending parser \
            implementation"]
fn prepare_returns_expanded_dataset() {
  let parser = Parser::new();
  let dataset = parser
    .prepare("Hello, world!", true);

  let expected: Vec<Vec<String>> = vec![
    PREPARED_LINES
      .iter()
      .map(|line| line.to_string())
      .collect(),
  ];

  assert_eq!(
    dataset.to_vec(),
    &expected,
    "parser.prepare should expand \
     tokens exactly as AnyStyle 1.x"
  );
}

#[test]
#[ignore = "pending parser \
            implementation"]
fn parse_returns_metadata_map() {
  let parser = Parser::new();
  let references = parser.parse(
    &[PEREC_REF],
    ParseFormat::Json
  );

  assert_eq!(
    references.len(),
    1,
    "Should return one reference"
  );

  let mut expected_fields =
    BTreeMap::new();
  expected_fields.insert(
    "title".into(),
    FieldValue::List(vec![
      "A Void".into(),
    ])
  );
  expected_fields.insert(
    "location".into(),
    FieldValue::List(vec![
      "London".into(),
    ])
  );
  expected_fields.insert(
    "publisher".into(),
    FieldValue::List(vec![
      "The Harvill Press".into(),
    ])
  );
  expected_fields.insert(
    "date".into(),
    FieldValue::List(vec![
      "1995".into(),
    ])
  );
  expected_fields.insert(
    "pages".into(),
    FieldValue::List(vec![
      "108".into(),
    ])
  );
  expected_fields.insert(
    "type".into(),
    FieldValue::Single("book".into())
  );

  let reference = &references[0].0;
  assert!(
    expected_fields.keys().all(|key| {
      reference.contains_key(key)
    }),
    "Expected parser.parse to \
     populate the documented fields"
  );
}

#[test]
#[ignore = "pending parser \
            implementation"]
fn label_handles_empty_lines() {
  let parser = Parser::new();
  assert!(parser.label("").is_empty());
  assert!(
    parser.label("\n").is_empty()
  );
  assert!(
    parser.label(" \n \n").is_empty()
  );
}

#[test]
#[ignore = "pending parser \
            implementation"]
fn label_outputs_all_expected_segment_types()
 {
  let parser = Parser::new();
  let sequences =
    parser.label(&format!(
      "{}\n{}",
      PEREC_REF, PEREC_REF
    ));

  let found: Vec<String> = sequences
    .iter()
    .flatten()
    .map(|token| token.label.clone())
    .collect();

  let unique_labels: Vec<_> = found
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();

  let expected_labels = [
    "author",
    "title",
    "location",
    "publisher",
    "date",
    "pages"
  ];
  for expected in expected_labels {
    assert!(
      unique_labels.contains(
        &expected.to_string()
      ),
      "label output should contain \
       {expected}"
    );
  }
}

#[test]
#[ignore = "pending parser \
            implementation"]
fn label_handles_unrecognizable_input()
{
  let parser = Parser::new();
  parser
    .label("@misc{70213094902020,\n");
  parser.label("\n doi ");
}
