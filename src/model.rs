use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::Result;
use serde::{
  Deserialize,
  Serialize
};

#[derive(
  Debug, Default, Serialize, Deserialize,
)]
pub struct ParserModel {
  datasets: HashMap<String, usize>
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
    sequences: usize
  ) {
    self.datasets.insert(
      path.display().to_string(),
      sequences
    );
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

#[derive(
  Debug, Default, Serialize, Deserialize,
)]
pub struct FinderModel {
  datasets: HashMap<String, usize>
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
}
