use crate::document::Document;

#[derive(Debug, Clone)]
pub struct Finder;

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
