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
const COMPLEX_REF_WITH_NUMBER: &str =
  "Smith, Alice. On heuristics for \
   mixing metadata. Lecture Notes \
   in Computer Science, 4050. Journal \
   of Testing. Edited by Doe, J. \
   (Note: Preprint release). \
   doi:10.1000/test https://example.org.";
const TRANSLATOR_REF: &str =
  "Roe, Jane. Title. Translated by \
   Doe, J. ISBN 978-1-2345-6789-0 \
   ISSN 1234-5678.";

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
fn csl_formatter_outputs_name_objects()
{
  let parser = Parser::new();
  let references = parser.parse(
    &[PEREC_REF],
    ParseFormat::Csl
  );
  let formatter = Format::new();
  let csl =
    formatter.to_csl(&references);

  assert!(
    csl.contains(
      "\"author\":[{\"family\":\"\
       Perec\",\"given\":\"Georges\"}]"
    ),
    "CSL output should emit \
     structured name objects"
  );
}

#[test]
fn csl_formatter_includes_collection_numbers_and_translators()
 {
  let parser = Parser::new();
  let references = parser.parse(
    &[
      COMPLEX_REF_WITH_NUMBER,
      TRANSLATOR_REF
    ],
    ParseFormat::Csl
  );
  let formatter = Format::new();
  let csl =
    formatter.to_csl(&references);

  assert!(
    csl.contains(
      "\"collection-number\":\"4050\""
    ),
    "CSL output should include \
     collection numbers"
  );
  assert!(
    csl.contains(
      "\"translator\":[{\"literal\":\"\
       Translated by Doe\"}]"
    ),
    "CSL output should include \
     translator names"
  );
  assert!(
    csl.contains(
      "\"isbn\":\"978-1-2345-6789-0\""
    ),
    "CSL output should include ISBN"
  );
  assert!(
    csl.contains(
      "\"issn\":\"1234-5678\""
    ),
    "CSL output should include ISSN"
  );
}

#[test]
fn bibtex_uses_publisher_place_when_missing_location()
 {
  let formatter = Format::new();
  let mut reference = Reference::new();
  reference.insert(
    "publisher-place",
    FieldValue::List(vec![
      "Paris".into(),
    ])
  );
  reference.insert(
    "type",
    FieldValue::Single("book".into())
  );
  let bibtex =
    formatter.to_bibtex(&[reference]);
  assert!(
    bibtex
      .contains("address = {Paris}"),
    "BibTeX should map \
     publisher-place to address"
  );
}

#[test]
fn csl_includes_issued_page_and_volume_issue()
 {
  let formatter = Format::new();
  let mut reference = Reference::new();
  reference.insert(
    "date",
    FieldValue::List(vec![
      "2020-12-05".into(),
    ])
  );
  reference.insert(
    "pages",
    FieldValue::List(vec![
      "12-34".into(),
    ])
  );
  reference.insert(
    "volume",
    FieldValue::List(vec!["42".into()])
  );
  reference.insert(
    "issue",
    FieldValue::List(vec!["3".into()])
  );
  let csl =
    formatter.to_csl(&[reference]);

  assert!(
    csl.contains(
      "\"issued\":{\"date-parts\":\
       [[2020,12,5]]}"
    ),
    "CSL output should emit issued \
     date-parts"
  );
  assert!(
    csl.contains("\"page\":\"12-34\""),
    "CSL output should emit page \
     ranges"
  );
  assert!(
    csl.contains(
      "\"page-first\":\"12\""
    ),
    "CSL output should emit page-first"
  );
  assert!(
    csl.contains("\"volume\":\"42\""),
    "CSL output should emit volume"
  );
  assert!(
    csl.contains("\"issue\":\"3\""),
    "CSL output should emit issue"
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
