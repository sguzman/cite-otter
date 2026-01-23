use cite_otter::format::{
  Format,
  ParseFormat
};
use cite_otter::parser::Parser;

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
  let formatter = Format;
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
  let formatter = Format;
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
  let formatter = Format;
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
