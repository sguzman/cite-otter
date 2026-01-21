use crate::parser::Reference;

#[derive(Debug, Clone, Copy)]
pub enum ParseFormat {
  Json,
  BibTeX,
  Csl
}

#[derive(Debug, Clone)]
pub struct Format;

impl Format {
  pub fn to_bibtex(
    &self,
    _references: &[Reference]
  ) -> String {
    todo!(
      "BibTeX formatting is pending \
       implementation"
    )
  }
}
