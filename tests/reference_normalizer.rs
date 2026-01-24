use std::fs;

use cite_otter::normalizer::abbreviations::AbbreviationMap;
use cite_otter::normalizer::container::Normalizer as ContainerNormalizer;
use cite_otter::normalizer::journal::Normalizer as JournalNormalizer;
use cite_otter::normalizer::location::Normalizer as LocationNormalizer;
use cite_otter::normalizer::names::Normalizer;
use serde_json::{
  Map,
  Value
};

#[test]
fn names_repeaters_resolve_to_previous_literal()
 {
  let normalizer = Normalizer::new();
  let repeat = "-----.,";

  let without_previous =
    normalizer.normalize(repeat, None);
  assert_eq!(
    without_previous
      .first()
      .map(|s| s.as_str()),
    Some("-----."),
    "Zero previous authors should \
     return the literal string"
  );

  let previous = ["X"];
  let with_previous = normalizer
    .normalize(repeat, Some(&previous));
  assert_eq!(
    with_previous
      .first()
      .map(|s| s.as_str()),
    Some("X"),
    "Repeaters should resolve to the \
     previous author when available"
  );
}

#[test]
fn location_normalizer_splits_location_and_publisher()
 {
  let normalizer =
    LocationNormalizer::new();
  let (location, publisher) =
    normalizer.normalize(
      "Chicago: Aldine Publishing Co."
    );

  assert_eq!(location, "Chicago");
  assert_eq!(
    publisher,
    Some(
      "Aldine Publishing Co.".into()
    )
  );
}

#[test]
fn container_normalizer_strips_prefixes()
 {
  let normalizer =
    ContainerNormalizer::new();
  let normalized = normalizer
    .normalize(
      "In Proceedings of Testing."
    );

  assert_eq!(
    normalized,
    "Proceedings of Testing"
  );
}

#[test]
fn journal_normalizer_expands_abbreviations()
 {
  let contents = fs::read_to_string(
    "tests/fixtures/abbrev-sample.txt"
  )
  .expect("abbrev fixture");
  let abbreviations =
    AbbreviationMap::load_from_str(
      &contents
    );
  let normalizer =
    JournalNormalizer::new();

  let mut map = Map::new();
  map.insert(
    "journal".into(),
    Value::Array(vec![Value::String(
      "J. Test.".into()
    )])
  );

  normalizer.normalize_with_abbrev(
    &mut map,
    &abbreviations
  );

  let container = map
    .get("container-title")
    .and_then(|value| value.as_array())
    .and_then(|array| array.first())
    .and_then(|value| value.as_str());
  assert_eq!(
    container,
    Some("Journal of Testing")
  );
  let article_type = map
    .get("type")
    .and_then(|value| value.as_str());
  assert_eq!(
    article_type,
    Some("article-journal")
  );
}
