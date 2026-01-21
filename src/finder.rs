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

  pub fn label(
    &self,
    input: &str
  ) -> Vec<Document> {
    if input.trim().is_empty() {
      return Vec::new();
    }

    vec![Document::from_text(input)]
  }
}
