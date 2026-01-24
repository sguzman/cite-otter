use std::collections::{
  BTreeMap,
  BTreeSet
};

use cite_otter::dictionary::{
  Dictionary,
  DictionaryAdapter,
  DictionaryCode
};
use cite_otter::format::ParseFormat;
use cite_otter::parser::{
  Author,
  FieldValue,
  Parser
};

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

const PEREC_REF_NO_COMMA: &str =
  "Georges Perec. A Void. London: The \
   Harvill Press, 1995. p.108.";

const PEREC_MULTI_YEAR_REF: &str =
  "Perec, Georges. A Void. London: \
   The Harvill Press, 1995/96. p.108.";

const COMPLEX_REF: &str =
  "Smith, Alice. On heuristics for \
   mixing metadata. Lecture Notes in \
   Computer Science, 4050. Journal of \
   Testing. Edited by Doe, J. \
   (Note: Preprint release). \
   doi:10.1000/test https://example.org.";

#[test]
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
    expected_fields.keys().all(
      |key: &String| {
        reference.contains_key(key)
      }
    ),
    "Expected parser.parse to \
     populate the documented fields"
  );
}

#[test]
fn parse_captures_collection_journal_editor_and_identifiers()
 {
  let parser = Parser::new();
  let references = parser.parse(
    &[COMPLEX_REF],
    ParseFormat::Json
  );

  let reference = &references[0].0;
  assert_list_field(
    reference,
    "collection-title",
    "Lecture Notes in Computer Science"
  );
  assert_list_field(
    reference,
    "journal",
    "Journal of Testing"
  );
  assert_list_field(
    reference,
    "editor",
    "Edited by Doe"
  );
  assert_list_field(
    reference,
    "note",
    "Note: Preprint release"
  );
  assert_list_field(
    reference,
    "doi",
    "doi:10.1000/test"
  );
  assert_list_field(
    reference,
    "url",
    "https://example.org"
  );
}

#[test]
fn parse_builds_structured_authors_for_variant_formats()
 {
  let parser = Parser::new();
  let references = parser.parse(
    &[PEREC_REF, PEREC_REF_NO_COMMA],
    ParseFormat::Json
  );

  let expected = Author {
    family: "Perec".into(),
    given:  "Georges".into()
  };

  for reference in references {
    let author_field = reference
      .fields()
      .get("author")
      .expect(
        "parser should always emit an \
         author"
      );

    let authors = match author_field {
      FieldValue::Authors(list) => list,
      other => panic!(
        "Expected FieldValue::Authors, got {other:?}"
      )
    };

    assert!(
      authors.first()
        == Some(&expected),
      "Each reference should \
       normalize author components \
       consistently"
    );
  }
}

#[test]
fn parse_uses_dictionary_for_type_resolution()
 {
  let mut dictionary =
    Dictionary::create(
      DictionaryAdapter::Memory
    )
    .open();
  dictionary
    .import_entries(vec![(
      "Nature".to_string(),
      DictionaryCode::Journal.bit()
    )])
    .expect("dictionary import");
  let parser =
    Parser::with_dictionary(dictionary);

  let references = parser.parse(
    &["Doe, J. Nature. 2020."],
    ParseFormat::Json
  );
  let reference = &references[0].0;
  let parsed = reference
    .get("type")
    .expect("type field");
  match parsed {
    | FieldValue::Single(value) => {
      assert_eq!(
        value, "article",
        "dictionary journal tag \
         should set article type"
      );
    }
    | _ => {
      panic!(
        "expected single type value"
      )
    }
  }
}

#[test]
fn parse_prefers_first_year_in_multi_year_tokens()
 {
  let parser = Parser::new();
  let references = parser.parse(
    &[PEREC_MULTI_YEAR_REF],
    ParseFormat::Json
  );

  let date_values = match references[0]
    .fields()
    .get("date")
  {
    | Some(FieldValue::List(
      values
    )) => values,
    | other => {
      panic!(
        "Expected list of date \
         tokens, got {other:?}"
      )
    }
  };

  let expected: Vec<String> =
    vec!["1995".into(), "1996".into()];
  assert_eq!(
    date_values, &expected,
    "Parser should normalize the \
     multi-year range"
  );
}

#[test]
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
    .collect::<BTreeSet<_>>()
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
fn label_handles_unrecognizable_input()
{
  let parser = Parser::new();
  parser
    .label("@misc{70213094902020,\n");
  parser.label("\n doi ");
}

fn assert_list_field(
  reference: &BTreeMap<
    String,
    FieldValue
  >,
  key: &str,
  expected: &str
) {
  match reference.get(key) {
    | Some(FieldValue::List(
      values
    )) => {
      assert_eq!(
        values
          .first()
          .map(|value| value.as_str()),
        Some(expected),
        "field {key} should contain \
         {expected}"
      );
    }
    | other => {
      panic!(
        "expected list value for \
         {key}, got {other:?}"
      )
    }
  }
}
