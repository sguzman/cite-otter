mod support;

use cite_otter::document::Document;
use support::fixture_path;

#[test]
#[ignore = "pending document \
            implementation"]
fn document_counts_pages_from_fixture()
{
  let doc = Document::open(
    fixture_path("phd.txt")
  );
  assert_eq!(
    doc.pages().len(),
    84,
    "Document should report 84 pages \
     for the published PhD fixture"
  );
}
