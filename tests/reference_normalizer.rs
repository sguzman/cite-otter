use cite_otter::normalizer::names::Normalizer;

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
