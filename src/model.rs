use std::collections::{
  HashMap,
  HashSet
};
use std::fs;
use std::path::Path;

use anyhow::Result;
use serde::{
  Deserialize,
  Serialize
};

#[derive(
  Debug, Serialize, Deserialize, Clone,
)]
#[serde(untagged)]
enum ParserStat {
  Count(usize),
  Full {
    sequences: usize,
    #[serde(default)]
    tokens:    usize
  }
}

impl ParserStat {
  fn sequences(&self) -> usize {
    match self {
      | Self::Count(count) => *count,
      | Self::Full {
        sequences,
        ..
      } => *sequences
    }
  }

  fn tokens(&self) -> usize {
    match self {
      | Self::Count(_) => 0,
      | Self::Full {
        tokens, ..
      } => *tokens
    }
  }

  fn from_counts(
    sequences: usize,
    tokens: usize
  ) -> Self {
    Self::Full {
      sequences,
      tokens
    }
  }
}

#[derive(
  Debug,
  Default,
  Serialize,
  Deserialize,
  Clone,
)]
pub struct ParserModel {
  datasets: HashMap<String, ParserStat>
}

impl ParserModel {
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
    path: &Path,
    sequences: usize,
    tokens: usize
  ) {
    self.datasets.insert(
      path.display().to_string(),
      ParserStat::from_counts(
        sequences, tokens
      )
    );
  }

  pub fn sequences(
    &self,
    path: &Path
  ) -> Option<usize> {
    self
      .datasets
      .get(&path.display().to_string())
      .map(ParserStat::sequences)
  }

  pub fn tokens(
    &self,
    path: &Path
  ) -> Option<usize> {
    self
      .datasets
      .get(&path.display().to_string())
      .map(ParserStat::tokens)
  }
}

#[derive(
  Debug,
  Default,
  Serialize,
  Deserialize,
  Clone,
)]
pub struct FinderModel {
  datasets:   HashMap<String, usize>,
  #[serde(default)]
  signatures: HashSet<String>
}

impl FinderModel {
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
    path: &Path,
    sequences: usize
  ) {
    self.datasets.insert(
      path.display().to_string(),
      sequences
    );
  }

  pub fn record_signature(
    &mut self,
    signature: String
  ) {
    self.signatures.insert(signature);
  }

  pub fn contains_signature(
    &self,
    signature: &str
  ) -> bool {
    self.signatures.contains(signature)
  }

  pub fn has_signatures(&self) -> bool {
    !self.signatures.is_empty()
  }

  pub fn sequences(
    &self,
    path: &Path
  ) -> Option<usize> {
    self
      .datasets
      .get(&path.display().to_string())
      .copied()
  }
}
