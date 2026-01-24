use std::fs;

use cite_otter::format::{
  Format,
  ParseFormat
};
use cite_otter::normalizer::{
  abbreviations::AbbreviationMap,
  NormalizationConfig
};
use cite_otter::parser::{
  FieldValue,
  Parser,
  Reference
};

const PEREC_REF: &str =
  "Perec, Georges. A Void. London: \
   The Harvill Press, 1995. p.108.";
const COMPLEX_REF: &str =
  "Smith, Alice. On heuristics for \
   mixing metadata. Lecture Notes \
   in Computer Science. Journal of \
   Testing. Edited by Doe, J. \
   (Note: Preprint release). \
   doi:10.1000/test https://example.org.";

#[test]
fn bibtex_formatter_round_trips_reference()
 {
  let parser = Parser::new();
  let references = parser.parse(
    &[PEREC_REF],
    ParseFormat::BibTeX
  );
  let formatter = Format::new();
  let bibtex =
    formatter.to_bibtex(&references);

  assert!(
    bibtex.contains("@book"),
    "BibTeX formatter should emit the \
     expected entry type"
  );
}

#[test]
fn bibtex_includes_container_series_and_doi()
 {
  let parser = Parser::new();
  let references = parser.parse(
    &[COMPLEX_REF],
    ParseFormat::BibTeX
  );
  let formatter = Format::new();
  let bibtex =
    formatter.to_bibtex(&references);

  assert!(
    bibtex.contains(
      "booktitle = {Journal of \
       Testing}"
    ),
    "BibTeX output should promote \
     journal metadata into booktitle"
  );
  assert!(
    bibtex.contains(
      "series = {Lecture Notes in \
       Computer Science}"
    ),
    "Series metadata should become \
     the BibTeX series field"
  );
  assert!(
    bibtex.contains(
      "doi = {doi:10.1000/test}"
    ),
    "DOI metadata should be exposed"
  );
}

#[test]
fn csl_formatter_outputs_enriched_json()
{
  let parser = Parser::new();
  let references = parser.parse(
    &[COMPLEX_REF],
    ParseFormat::Csl
  );
  let formatter = Format::new();
  let csl =
    formatter.to_csl(&references);

  assert!(
    csl.contains(
      "\"container-title\":\"Journal \
       of Testing\""
    ),
    "CSL output should report the \
     journal as container-title"
  );
  assert!(
    csl.contains(
      "\"doi\":\"doi:10.1000/test\""
    ),
    "CSL output should expose DOI"
  );
}

#[test]
fn formatter_expands_journal_abbrev() {
  let contents = fs::read_to_string(
    "tests/fixtures/abbrev-sample.txt"
  )
  .expect("abbrev fixture");
  let abbreviations =
    AbbreviationMap::load_from_str(
      &contents
    );
  let config =
    NormalizationConfig::default()
      .with_journal_abbrev(
        abbreviations
      );
  let formatter =
    Format::with_normalization(config);

  let mut reference = Reference::new();
  reference.insert(
    "journal",
    FieldValue::List(vec![
      "J. Test.".into(),
    ])
  );
  let csl =
    formatter.to_csl(&[reference]);

  assert!(
    csl.contains(
      "\"container-title\":\"Journal \
       of Testing\""
    ),
    "Formatter should expand journal \
     abbreviations"
  );
}

#[test]
fn formatter_expands_publisher_and_container_abbrev()
 {
  let publisher_text =
    fs::read_to_string(
      "tests/fixtures/\
       publisher-abbrev-sample.txt"
    )
    .expect("publisher fixture");
  let container_text =
    fs::read_to_string(
      "tests/fixtures/\
       container-abbrev-sample.txt"
    )
    .expect("container fixture");
  let config =
    NormalizationConfig::default()
      .with_publisher_abbrev(
        AbbreviationMap::load_from_str(
          &publisher_text
        )
      )
      .with_container_abbrev(
        AbbreviationMap::load_from_str(
          &container_text
        )
      );
  let formatter =
    Format::with_normalization(config);

  let mut reference = Reference::new();
  reference.insert(
    "publisher",
    FieldValue::List(vec![
      "Univ. Press".into(),
    ])
  );
  reference.insert(
    "container-title",
    FieldValue::List(vec![
      "Proc. Test.".into(),
    ])
  );
  let csl =
    formatter.to_csl(&[reference]);

  assert!(
    csl.contains(
      "\"publisher\":\"University \
       Press\""
    ),
    "Formatter should expand \
     publisher abbreviations"
  );
  assert!(
    csl.contains(
      "\"container-title\":\"\
       Proceedings of Testing\""
    ),
    "Formatter should expand \
     container abbreviations"
  );
}
