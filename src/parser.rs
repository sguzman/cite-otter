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

#[derive(Debug, Clone)]
pub struct Parser;

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
    _input: &str,
    _expand: bool
  ) -> ParsedDataset {
    todo!(
      "Parser preparation is pending \
       implementation"
    )
  }

  pub fn parse(
    &self,
    _refs: &[&str],
    _format: ParseFormat
  ) -> Vec<Reference> {
    todo!(
      "Parser parse output pending \
       implementation"
    )
  }

  pub fn label(
    &self,
    _input: &str
  ) -> Vec<Vec<TaggedToken>> {
    todo!(
      "Parser labeling is pending \
       implementation"
    )
  }
}

impl ParsedDataset {
  pub fn to_vec(
    &self
  ) -> &Vec<Vec<String>> {
    &self.0
  }
}
