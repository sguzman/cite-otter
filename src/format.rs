use serde_json::Value;

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

impl Default for Format {
  fn default() -> Self {
    Self::new()
  }
}

impl Format {
  pub fn new() -> Self {
    Self
  }

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

  pub fn to_json(
    &self,
    references: &[Reference]
  ) -> String {
    serde_json::to_string_pretty(
      references
    )
    .unwrap_or_else(|_| "[]".into())
  }

  pub fn to_value(
    &self,
    references: &[Reference]
  ) -> Value {
    serde_json::to_value(references)
      .unwrap_or(Value::Null)
  }
}
