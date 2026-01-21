mod support;

use std::fs;

use cite_otter::finder::Finder;
use support::fixture_path;

#[test]
#[ignore = "pending finder \
            implementation"]
fn finder_detects_references_in_a_document()
 {
  let path = fixture_path("phd.txt");
  let text = fs::read_to_string(path)
    .expect(
      "fixture should be readable"
    );
  let finder = Finder::new();
  let sequences = finder.label(&text);

  assert!(
    !sequences.is_empty(),
    "finder should locate at least \
     one reference sequence"
  );
}
