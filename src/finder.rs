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
    _input: &str
  ) -> Vec<Document> {
    todo!(
      "Finder labeling is pending \
       implementation"
    )
  }
}
