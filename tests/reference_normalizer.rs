use std::fs;

use cite_otter::normalizer::abbreviations::AbbreviationMap;
use cite_otter::normalizer::container::Normalizer as ContainerNormalizer;
use cite_otter::normalizer::journal::Normalizer as JournalNormalizer;
use cite_otter::normalizer::location::Normalizer as LocationNormalizer;
use cite_otter::normalizer::names::Normalizer;
use cite_otter::normalizer::NormalizationConfig;
use serde_json::{
  Map,
  Value
};
use tempfile::tempdir;

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

#[test]
fn normalization_config_expands_publisher_and_container()
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

  let mut map = Map::new();
  map.insert(
    "publisher".into(),
    Value::String("Univ. Press".into())
  );
  map.insert(
    "container-title".into(),
    Value::Array(vec![Value::String(
      "Proc. Test.".into()
    )])
  );

  config.apply_to_map(&mut map);

  let publisher = map
    .get("publisher")
    .and_then(|value| value.as_str());
  assert_eq!(
    publisher,
    Some("University Press")
  );

  let container = map
    .get("container-title")
    .and_then(|value| value.as_array())
    .and_then(|array| array.first())
    .and_then(|value| value.as_str());
  assert_eq!(
    container,
    Some("Proceedings of Testing")
  );
}

#[test]
fn normalization_config_loads_from_dir()
{
  let dir = std::path::Path::new(
    "tests/fixtures/normalization-dir"
  );
  let config =
    NormalizationConfig::load_from_dir(
      dir
    )
    .expect("load normalization dir");

  let mut map = Map::new();
  map.insert(
    "journal".into(),
    Value::Array(vec![Value::String(
      "J. Test.".into()
    )])
  );
  map.insert(
    "publisher".into(),
    Value::String("Univ. Press".into())
  );
  map.insert(
    "container-title".into(),
    Value::Array(vec![Value::String(
      "Proc. Test.".into()
    )])
  );

  config.apply_to_map(&mut map);

  let container_values = map
    .get("container-title")
    .and_then(|value| value.as_array())
    .map(|array| {
      array
        .iter()
        .filter_map(|value| {
          value.as_str()
        })
        .collect::<Vec<_>>()
    })
    .unwrap_or_default();
  assert!(
    container_values
      .contains(&"Journal of Testing"),
    "expected expanded journal"
  );
  assert!(
    container_values.contains(
      &"Proceedings of Testing"
    ),
    "expected expanded container title"
  );
  let publisher = map
    .get("publisher")
    .and_then(|value| value.as_str());
  assert_eq!(
    publisher,
    Some("University Press")
  );
}

#[test]
fn normalization_config_handles_missing_assets()
 {
  let dir =
    tempdir().expect("temp dir");
  fs::write(
    dir
      .path()
      .join("journal-abbrev.txt"),
    "J. Test.\tJournal of Testing"
  )
  .expect("write journal abbrev");

  let config =
    NormalizationConfig::load_from_dir(
      dir.path()
    )
    .expect("load normalization dir");

  let mut map = Map::new();
  map.insert(
    "journal".into(),
    Value::Array(vec![Value::String(
      "J. Test.".into()
    )])
  );
  map.insert(
    "publisher".into(),
    Value::String("Univ. Press".into())
  );
  map.insert(
    "container-title".into(),
    Value::Array(vec![Value::String(
      "Proc. Test.".into()
    )])
  );

  config.apply_to_map(&mut map);

  let container_values = map
    .get("container-title")
    .and_then(Value::as_array)
    .map(|array| {
      array
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>()
    })
    .unwrap_or_default();
  assert!(
    container_values
      .contains(&"Journal of Testing"),
    "expected expanded journal entry"
  );
  assert!(
    container_values
      .contains(&"Proc. Test."),
    "missing container abbrev should \
     leave container-title unchanged"
  );
  let publisher = map
    .get("publisher")
    .and_then(Value::as_str);
  assert_eq!(
    publisher,
    Some("Univ. Press"),
    "missing publisher abbrev should \
     keep original publisher"
  );
}

#[test]
fn normalization_config_applies_locale_overrides()
 {
  let dir =
    tempdir().expect("temp dir");
  fs::write(
    dir
      .path()
      .join("language-locale.txt"),
    "en\ten-US\n"
  )
  .expect("write language locale");
  fs::write(
    dir
      .path()
      .join("script-locale.txt"),
    "Latin\tLatn\n"
  )
  .expect("write script locale");

  let config =
    NormalizationConfig::load_from_dir(
      dir.path()
    )
    .expect("load normalization dir");

  let mut map = Map::new();
  map.insert(
    "language".into(),
    Value::String("en".into())
  );
  map.insert(
    "scripts".into(),
    Value::Array(vec![Value::String(
      "Latin".into()
    )])
  );

  config.apply_to_map(&mut map);

  let language = map
    .get("language")
    .and_then(Value::as_str);
  assert_eq!(
    language,
    Some("en-US"),
    "locale overrides should map \
     language codes"
  );

  let scripts = map
    .get("scripts")
    .and_then(Value::as_array)
    .and_then(|array| {
      array
        .first()
        .and_then(Value::as_str)
    });
  assert_eq!(
    scripts,
    Some("Latn"),
    "locale overrides should map \
     script names"
  );
}

#[test]
fn normalization_config_prefers_manual_locale_overrides()
 {
  let dir = std::path::Path::new(
    "tests/fixtures/normalization-any"
  );
  let config =
    NormalizationConfig::load_from_dir(
      dir
    )
    .expect("load normalization dir");
  let overrides =
    AbbreviationMap::load_from_str(
      "en\ten-GB"
    );
  let config = config
    .with_language_locale(overrides);

  let mut map = Map::new();
  map.insert(
    "language".into(),
    Value::String("en".into())
  );
  config.apply_to_map(&mut map);

  let language = map
    .get("language")
    .and_then(Value::as_str);
  assert_eq!(
    language,
    Some("en-GB"),
    "manual locale overrides should \
     take precedence"
  );
}

#[test]
fn normalization_any_assets_load_locale_mappings()
 {
  let config =
    NormalizationConfig::load_from_dir(
      std::path::Path::new(
        "tests/fixtures/\
         normalization-any"
      )
    )
    .expect("normalization assets");
  let mut map = Map::new();
  map.insert(
    "language".into(),
    Value::String("en".into())
  );
  map.insert(
    "scripts".into(),
    Value::String("Latin".into())
  );
  config.apply_to_map(&mut map);
  let language = map
    .get("language")
    .and_then(Value::as_str);
  assert_eq!(
    language,
    Some("en-US"),
    "normalization assets should map \
     language codes"
  );
  let scripts = map
    .get("scripts")
    .and_then(Value::as_str);
  assert_eq!(
    scripts,
    Some("Latn"),
    "normalization assets should map \
     script names"
  );
}

#[test]
fn abbreviations_last_entry_wins() {
  let abbreviations =
    AbbreviationMap::load_from_str(
      "J. Test.\tOld Journal\nJ. \
       Test.\tNew Journal"
    );
  assert_eq!(
    abbreviations.expand("J. Test."),
    "New Journal",
    "last abbreviation entry should \
     win for duplicates"
  );
}

#[test]
fn normalization_any_assets_are_populated()
 {
  let dir = std::path::Path::new(
    "tests/fixtures/normalization-any"
  );
  let journal =
    AbbreviationMap::load_from_file(
      &dir.join("journal-abbrev.txt")
    )
    .expect("journal abbrev");
  let container =
    AbbreviationMap::load_from_file(
      &dir.join("container-abbrev.txt")
    )
    .expect("container abbrev");
  let publisher =
    AbbreviationMap::load_from_file(
      &dir.join("publisher-abbrev.txt")
    )
    .expect("publisher abbrev");
  let language =
    AbbreviationMap::load_from_file(
      &dir.join("language-locale.txt")
    )
    .expect("language locale");
  let scripts =
    AbbreviationMap::load_from_file(
      &dir.join("script-locale.txt")
    )
    .expect("script locale");

  assert!(
    !journal.is_empty(),
    "journal abbreviations should be \
     populated"
  );
  assert!(
    !container.is_empty(),
    "container abbreviations should \
     be populated"
  );
  assert!(
    !publisher.is_empty(),
    "publisher abbreviations should \
     be populated"
  );
  assert!(
    !language.is_empty(),
    "language locale mapping should \
     be populated"
  );
  assert!(
    !scripts.is_empty(),
    "script locale mapping should be \
     populated"
  );
}
