use std::collections::HashSet;
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
  signatures: HashSet<String>
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
    self.signatures.insert(signature);
  }

  pub fn contains(
    &self,
    signature: &str
  ) -> bool {
    self.signatures.contains(signature)
  }

  pub fn has_signatures(&self) -> bool {
    !self.signatures.is_empty()
  }
}
