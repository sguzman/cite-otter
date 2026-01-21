use std::collections::BTreeMap;

use crate::format::ParseFormat;

#[derive(Debug, Clone)]
pub struct TaggedToken {
  pub token: String,
  pub label: String
}

#[derive(Debug, Clone)]
pub enum FieldValue {
  Single(String),
  List(Vec<String>)
}

#[derive(Debug, Clone)]
pub struct Reference(
  pub BTreeMap<String, FieldValue>
);

impl Reference {
  pub fn new() -> Self {
    Self(BTreeMap::new())
  }

  pub fn insert(
    &mut self,
    key: impl Into<String>,
    value: FieldValue
  ) {
    self.0.insert(key.into(), value);
  }

  pub fn fields(
    &self
  ) -> &BTreeMap<String, FieldValue> {
    &self.0
  }
}

impl Default for Reference {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug, Clone)]
pub struct Parser;

impl Default for Parser {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug, Clone)]
pub struct ParsedDataset(
  pub Vec<Vec<String>>
);

impl Parser {
  pub fn new() -> Self {
    Self
  }

  pub fn prepare(
    &self,
    input: &str,
    expand: bool
  ) -> ParsedDataset {
    let sequences = input
      .lines()
      .filter(|line| {
        !line.trim().is_empty()
      })
      .map(|line| {
        line
          .split_whitespace()
          .map(|token| {
            if expand {
              format!(
                "{} {}",
                token,
                token.len()
              )
            } else {
              token.to_string()
            }
          })
          .collect::<Vec<_>>()
      })
      .collect::<Vec<_>>();

    ParsedDataset(sequences)
  }

  pub fn label(
    &self,
    input: &str
  ) -> Vec<Vec<TaggedToken>> {
    self
      .prepare(input, true)
      .0
      .iter()
      .map(|sequence| {
        sequence
          .iter()
          .map(|token| {
            TaggedToken {
              token: token.clone(),
              label: "unknown".into()
            }
          })
          .collect::<Vec<_>>()
      })
      .collect::<Vec<_>>()
  }

  pub fn parse(
    &self,
    refs: &[&str],
    _format: ParseFormat
  ) -> Vec<Reference> {
    refs
      .iter()
      .map(|reference| {
        let mut mapped =
          Reference::new();
        mapped.insert(
          "title",
          FieldValue::List(vec![
            extract_title(reference),
          ])
        );
        mapped.insert(
          "type",
          FieldValue::Single(
            resolve_type(reference)
          )
        );
        mapped.insert(
          "date",
          FieldValue::List(vec![
            extract_year(reference)
              .unwrap_or_default(),
          ])
        );
        mapped
      })
      .collect()
  }
}

impl ParsedDataset {
  pub fn to_vec(
    &self
  ) -> &Vec<Vec<String>> {
    &self.0
  }
}

fn extract_title(
  reference: &str
) -> String {
  reference
    .split('.')
    .nth(1)
    .unwrap_or("")
    .trim()
    .to_string()
}

fn extract_year(
  reference: &str
) -> Option<String> {
  reference
    .split_whitespace()
    .find(|token| {
      token
        .chars()
        .filter(|c| c.is_ascii_digit())
        .count()
        >= 3
    })
    .map(|token| {
      token
        .trim_matches(|c: char| {
          !c.is_ascii_digit()
        })
        .to_string()
    })
}

fn resolve_type(
  reference: &str
) -> String {
  if reference
    .to_lowercase()
    .contains("journal")
  {
    "article".into()
  } else {
    "book".into()
  }
}
