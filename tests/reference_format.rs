use cite_otter::format::{
  Format,
  ParseFormat
};
use cite_otter::parser::Parser;

const PEREC_REF: &str =
  "Perec, Georges. A Void. London: \
   The Harvill Press, 1995. p.108.";

#[test]
#[ignore = "pending format \
            implementation"]
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
