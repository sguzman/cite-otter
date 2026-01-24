use cite_otter::language::{
  detect_language,
  detect_scripts
};

#[test]
fn detect_language_returns_en_for_english()
 {
  let language = detect_language(
    "This is English text about \
     citations and formats."
  );
  assert_eq!(
    language, "en",
    "language detection should \
     resolve English text"
  );
}

#[test]
fn detect_scripts_captures_multiple_scripts()
 {
  let scripts =
    detect_scripts("Hello Καλημέρα");
  assert!(
    scripts
      .contains(&"Latin".to_string()),
    "scripts should include Latin"
  );
  assert!(
    scripts
      .contains(&"Greek".to_string()),
    "scripts should include Greek"
  );
  assert!(
    scripts
      .contains(&"Common".to_string()),
    "scripts should include Common"
  );
}
