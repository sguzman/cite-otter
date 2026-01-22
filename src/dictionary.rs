#[derive(Debug, Clone, Copy)]
pub enum DictionaryAdapter {
  Memory,
  Redis,
  Lmdb,
  Gdbm
}

#[derive(
  Debug, Clone, Copy, PartialEq, Eq,
)]
pub enum DictionaryCode {
  Place
}

#[derive(Debug)]
pub struct Dictionary {
  adapter: DictionaryAdapter
}

impl Dictionary {
  pub fn create(
    adapter: DictionaryAdapter
  ) -> Self {
    Self {
      adapter
    }
  }

  pub fn open(self) -> Self {
    self
  }

  pub fn lookup(
    &self,
    term: &str
  ) -> Vec<DictionaryCode> {
    let normalized =
      term.to_lowercase();
    if PLACE_NAMES.iter().any(|&name| {
      normalized.contains(name)
    }) {
      vec![DictionaryCode::Place]
    } else {
      Vec::new()
    }
  }

  pub fn adapter(
    &self
  ) -> DictionaryAdapter {
    self.adapter
  }
}

static PLACE_NAMES: &[&str] =
  &["philippines", "italy"];
