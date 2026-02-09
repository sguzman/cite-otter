use std::collections::BTreeMap;

use serde::Serialize;

#[derive(Debug, Clone)]
pub struct TaggedToken {
  pub token: String,
  pub label: String
}

#[derive(
  Debug, Clone, Serialize, PartialEq, Eq,
)]
pub struct Author {
  pub family: String,
  pub given:  String
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum FieldValue {
  Single(String),
  List(Vec<String>),
  Authors(Vec<Author>)
}

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
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

  pub fn from_map(
    map: BTreeMap<String, FieldValue>
  ) -> Self {
    Self(map)
  }
}

impl Default for Reference {
  fn default() -> Self {
    Self::new()
  }
}
