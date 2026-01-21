use crate::parser::{
  FieldValue,
  Reference
};

#[derive(Debug, Clone)]
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
    references: &[Reference]
  ) -> String {
    references
      .iter()
      .enumerate()
      .map(|(idx, reference)| {
        let title = reference
          .fields()
          .get("title")
          .and_then(|value| {
            match value {
              | FieldValue::List(
                list
              ) => list.first(),
              | FieldValue::Single(
                value
              ) => Some(value)
            }
          })
          .cloned()
          .unwrap_or_else(|| {
            format!("cite-{:03}", idx)
          });

        format!(
          "@book{{citeotter{idx},\n  \
           title = {{{title}}}\n}}"
        )
      })
      .collect::<Vec<_>>()
      .join("\n\n")
  }
}
