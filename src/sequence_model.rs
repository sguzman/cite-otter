use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::Result;
use serde::{
  Deserialize,
  Serialize
};

#[derive(
  Debug,
  Clone,
  Default,
  Serialize,
  Deserialize,
)]
pub struct SequenceModel {
  counts: HashMap<String, usize>,
  total:  usize
}

impl SequenceModel {
  pub fn load(
    path: &Path
  ) -> Result<Self> {
    if path.exists() {
      let bytes = fs::read(path)?;
      Ok(serde_json::from_slice(
        &bytes
      )?)
    } else {
      Ok(Self::default())
    }
  }

  pub fn save(
    &self,
    path: &Path
  ) -> Result<()> {
    if let Some(parent) = path.parent()
    {
      fs::create_dir_all(parent)?;
    }
    let bytes =
      serde_json::to_vec_pretty(self)?;
    fs::write(path, bytes)?;
    Ok(())
  }

  pub fn record(
    &mut self,
    signature: String
  ) {
    *self
      .counts
      .entry(signature)
      .or_insert(0) += 1;
    self.total += 1;
  }

  pub fn count(
    &self,
    signature: &str
  ) -> usize {
    self
      .counts
      .get(signature)
      .copied()
      .unwrap_or(0)
  }

  pub fn has_signatures(&self) -> bool {
    !self.counts.is_empty()
  }

  pub fn should_match(
    &self,
    signature: &str,
    min_occurrences: usize
  ) -> bool {
    self.count(signature)
      >= min_occurrences
  }

  pub fn total(&self) -> usize {
    self.total
  }
}
