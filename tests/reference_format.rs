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
mod support;
use support::assert_snapshot_eq;

#[test]
fn format_diff_writes_report_for_mismatch()
 {
  let tmp =
    std::path::Path::new("target")
      .join("reports")
      .join("format-diff.txt");
  let _ = std::fs::remove_file(&tmp);
  let result =
    std::panic::catch_unwind(|| {
      assert_snapshot_eq(
        "diff fixture",
        "actual",
        "expected"
      );
    });
  assert!(result.is_err());
  assert!(
    tmp.exists(),
    "diff report should be written"
  );
}

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
const DERRIDA_REF: &str =
  "Derrida, J. (c.1967). L’écriture \
   et la différence (1 éd.). Paris: \
   Éditions du Seuil.";

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
fn bibtex_maps_article_journal_types() {
  let parser = Parser::new();
  let references = parser.parse(
    &[COMPLEX_REF],
    ParseFormat::BibTeX
  );
  let formatter = Format::new();
  let bibtex =
    formatter.to_bibtex(&references);

  assert!(
    bibtex.contains("@article"),
    "BibTeX formatter should map \
     article-journal types to article \
     entries"
  );
}

#[test]
fn bibtex_includes_container_series_and_doi()
 {
  let parser = Parser::new();
  let references = parser.parse(
    &[
      COMPLEX_REF_WITH_NUMBER,
      TRANSLATOR_REF
    ],
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
    bibtex.contains("number = {4050}"),
    "Series number should map to \
     BibTeX number"
  );
  assert!(
    bibtex.contains(
      "doi = {doi:10.1000/test}"
    ),
    "DOI metadata should be exposed"
  );
  assert!(
    bibtex.contains(
      "isbn = {978-1-2345-6789-0}"
    ),
    "ISBN metadata should be exposed"
  );
  assert!(
    bibtex
      .contains("issn = {1234-5678}"),
    "ISSN metadata should be exposed"
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
      "\"DOI\":\"doi:10.1000/test\""
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
      "\"issued\":\"2020-12-05\""
    ),
    "CSL output should emit issued \
     date strings"
  );
  assert!(
    csl.contains("\"page\":\"12-34\""),
    "CSL output should emit page \
     ranges"
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
fn csl_includes_circa_for_approximate_dates()
 {
  let parser = Parser::new();
  let references = parser.parse(
    &[DERRIDA_REF],
    ParseFormat::Csl
  );
  let formatter = Format::new();
  let csl =
    formatter.to_csl(&references);

  assert!(
    csl
      .contains("\"issued\":\"1967~\""),
    "CSL output should include circa \
     date strings"
  );
}

#[test]
fn bibtex_includes_edition_from_parser()
{
  let parser = Parser::new();
  let references = parser.parse(
    &[DERRIDA_REF],
    ParseFormat::BibTeX
  );
  let formatter = Format::new();
  let bibtex =
    formatter.to_bibtex(&references);

  assert!(
    bibtex.contains("edition = {1}"),
    "BibTeX output should include \
     edition"
  );
}

#[test]
fn bibtex_maps_date_to_year() {
  let parser = Parser::new();
  let references = parser.parse(
    &[PEREC_REF],
    ParseFormat::BibTeX
  );
  let formatter = Format::new();
  let bibtex =
    formatter.to_bibtex(&references);

  assert!(
    bibtex.contains("date = {1995}"),
    "BibTeX output should keep date \
     field"
  );
}

#[test]
fn bibtex_maps_article_issue_to_number()
{
  let formatter = Format::new();
  let mut reference = Reference::new();
  reference.insert(
    "type",
    FieldValue::Single(
      "article".into()
    )
  );
  reference.insert(
    "issue",
    FieldValue::List(vec!["7".into()])
  );
  let bibtex =
    formatter.to_bibtex(&[reference]);

  assert!(
    bibtex.contains("number = {7}"),
    "BibTeX should map issue to \
     number for article entries"
  );
  assert!(
    !bibtex.contains("issue = {7}"),
    "BibTeX output should not retain \
     issue when mapping to number"
  );
}

#[test]
fn bibtex_maps_report_and_conference_types()
 {
  let formatter = Format::new();
  let mut report = Reference::new();
  report.insert(
    "type",
    FieldValue::Single("report".into())
  );
  report.insert(
    "publisher",
    FieldValue::List(vec![
      "Test Institute".into(),
    ])
  );
  let report_bibtex =
    formatter.to_bibtex(&[report]);
  assert!(
    report_bibtex
      .contains("@techreport"),
    "BibTeX formatter should map \
     report types to techreport"
  );
  assert!(
    report_bibtex.contains(
      "institution = {Test Institute}"
    ),
    "Techreport entries should map \
     publisher into institution"
  );

  let mut conference = Reference::new();
  conference.insert(
    "type",
    FieldValue::Single(
      "paper-conference".into()
    )
  );
  conference.insert(
    "container-title",
    FieldValue::List(vec![
      "Proceedings of Testing".into(),
    ])
  );
  let conf_bibtex =
    formatter.to_bibtex(&[conference]);
  assert!(
    conf_bibtex
      .contains("@inproceedings"),
    "BibTeX formatter should map \
     paper-conference types to \
     inproceedings"
  );
  assert!(
    conf_bibtex.contains(
      "booktitle = {Proceedings of \
       Testing}"
    ),
    "Conference entries should map \
     container-title into booktitle"
  );
}

#[test]
fn csl_includes_type_note_genre_edition_and_language()
 {
  let formatter = Format::new();
  let mut reference = Reference::new();
  reference.insert(
    "type",
    FieldValue::Single("book".into())
  );
  reference.insert(
    "note",
    FieldValue::List(vec![
      "Special issue".into(),
    ])
  );
  reference.insert(
    "genre",
    FieldValue::List(vec![
      "Report".into(),
    ])
  );
  reference.insert(
    "edition",
    FieldValue::List(vec![
      "2nd".into(),
    ])
  );

  let csl =
    formatter.to_csl(&[reference]);

  assert!(
    csl.contains("\"type\":\"book\""),
    "CSL output should include type"
  );
  assert!(
    csl.contains(
      "\"note\":\"Special issue\""
    ),
    "CSL output should include note"
  );
  assert!(
    csl
      .contains("\"genre\":\"Report\""),
    "CSL output should include genre"
  );
  assert!(
    csl.contains("\"edition\":\"2nd\""),
    "CSL output should include edition"
  );
}

#[test]
fn csl_includes_scripts_array() {
  let formatter = Format::new();
  let mut reference = Reference::new();
  reference.insert(
    "scripts",
    FieldValue::List(vec![
      "Latin".into(),
      "Common".into(),
    ])
  );

  let csl =
    formatter.to_csl(&[reference]);

  assert!(
    !csl.contains("\"scripts\""),
    "CSL output should omit scripts"
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

#[test]
fn format_outputs_match_parity_snapshots()
 {
  let refs_text = fs::read_to_string(
    "tests/fixtures/format/refs.txt"
  )
  .expect("read format refs");
  let references = refs_text
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty())
    .map(|line| line.to_string())
    .collect::<Vec<_>>();
  let ref_slices = references
    .iter()
    .map(|line| line.as_str())
    .collect::<Vec<_>>();

  let parser = Parser::new();
  let parsed = parser.parse(
    &ref_slices,
    ParseFormat::Json
  );
  let formatter = Format::new();

  let csl_output =
    formatter.to_csl(&parsed);
  let expected_csl =
    fs::read_to_string(
      "tests/fixtures/format/csl.txt"
    )
    .expect("read expected CSL");
  assert_snapshot_eq(
    "sample:csl",
    &csl_output,
    &expected_csl
  );

  let bibtex_output =
    formatter.to_bibtex(&parsed);
  let expected_bibtex =
    fs::read_to_string(
      "tests/fixtures/format/bibtex.\
       txt"
    )
    .expect("read expected BibTeX");
  assert_snapshot_eq(
    "sample:bibtex",
    &bibtex_output,
    &expected_bibtex
  );
}

#[test]
fn format_core_outputs_match_snapshots()
{
  let refs_text = fs::read_to_string(
    "tests/fixtures/format/core-refs.\
     txt"
  )
  .expect("read core refs");
  let references = refs_text
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty())
    .map(|line| line.to_string())
    .collect::<Vec<_>>();
  let ref_slices = references
    .iter()
    .map(|line| line.as_str())
    .collect::<Vec<_>>();

  let parser = Parser::new();
  let parsed = parser.parse(
    &ref_slices,
    ParseFormat::Json
  );
  let formatter = Format::new();

  let csl_output =
    formatter.to_csl(&parsed);
  let expected_csl =
    fs::read_to_string(
      "tests/fixtures/format/core-csl.\
       txt"
    )
    .expect("read core CSL");
  assert_snapshot_eq(
    "core:csl",
    &csl_output,
    &expected_csl
  );

  let bibtex_output =
    formatter.to_bibtex(&parsed);
  let expected_bibtex =
    fs::read_to_string(
      "tests/fixtures/format/\
       core-bibtex.txt"
    )
    .expect("read core BibTeX");
  assert_snapshot_eq(
    "core:bibtex",
    &bibtex_output,
    &expected_bibtex
  );
}
