use crate::document::Document;

#[derive(Debug, Clone)]
pub struct Finder;

impl Default for Finder {
  fn default() -> Self {
    Self::new()
  }
}

impl Finder {
  pub fn new() -> Self {
    Self
  }

  pub fn segments(
    input: &str
  ) -> Vec<String> {
    split_into_references(input)
  }

  pub fn label(
    &self,
    input: &str
  ) -> Vec<Document> {
    let candidates =
      Self::segments(input);
    if candidates.is_empty() {
      return vec![Document::from_text(
        input
      )];
    }

    candidates
      .into_iter()
      .map(|segment| {
        Document::from_text(&segment)
      })
      .collect()
  }
}

fn split_into_references(
  input: &str
) -> Vec<String> {
  input
    .split("\n\n")
    .map(str::trim)
    .filter(|seg| {
      !seg.is_empty()
        && seg
          .chars()
          .any(|c| c.is_ascii_digit())
        && seg.len() > 20
    })
    .map(|seg| seg.to_string())
    .collect::<Vec<_>>()
}
