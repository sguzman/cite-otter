use std::collections::BTreeSet;

use super::tokenize::normalize_token;
use crate::parser::field_tokens::FieldTokens;

fn matches_field(
  normalized: &str,
  values: &BTreeSet<String>,
) -> bool {
  values.iter().any(|value| {
    !value.is_empty() && normalized.contains(value)
  })
}

pub(crate) fn tag_token(
  token: &str,
  context: &FieldTokens,
) -> String {
  let original = token.trim();
  let lower = original.to_lowercase();
  let normalized = normalize_token(original);

  if matches_field(&normalized, &context.identifier)
    || lower.contains("doi")
    || lower.starts_with("http")
    || lower.starts_with("www")
    || lower.contains("urn")
  {
    "identifier".into()
  } else if matches_field(&normalized, &context.author) {
    "author".into()
  } else if matches_field(&normalized, &context.title) {
    "title".into()
  } else if matches_field(&normalized, &context.journal) {
    "journal".into()
  } else if matches_field(&normalized, &context.container) {
    "container-title".into()
  } else if matches_field(&normalized, &context.location) {
    "location".into()
  } else if matches_field(&normalized, &context.publisher) {
    "publisher".into()
  } else if matches_field(&normalized, &context.collection) {
    "collection-title".into()
  } else if matches_field(&normalized, &context.date) {
    "date".into()
  } else if matches_field(&normalized, &context.editor) {
    "editor".into()
  } else if matches_field(&normalized, &context.translator) {
    "translator".into()
  } else if matches_field(&normalized, &context.note) {
    "note".into()
  } else if matches_field(&normalized, &context.pages) {
    "pages".into()
  } else if matches_field(&normalized, &context.volume) {
    "volume".into()
  } else if matches_field(&normalized, &context.issue) {
    "issue".into()
  } else if matches_field(&normalized, &context.genre) {
    "genre".into()
  } else if matches_field(&normalized, &context.edition) {
    "edition".into()
  } else {
    "other".into()
  }
}
