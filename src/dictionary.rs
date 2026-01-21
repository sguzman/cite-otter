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
    _term: &str
  ) -> Vec<DictionaryCode> {
    match self.adapter {
      | DictionaryAdapter::Memory => {
        vec![DictionaryCode::Place]
      }
      | DictionaryAdapter::Redis => {
        vec![DictionaryCode::Place]
      }
      | DictionaryAdapter::Lmdb => {
        vec![DictionaryCode::Place]
      }
      | DictionaryAdapter::Gdbm => {
        vec![DictionaryCode::Place]
      }
    }
  }
}
