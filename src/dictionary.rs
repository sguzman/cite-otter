use std::path::{
  Path,
  PathBuf
};

use anyhow::{
  Context,
  Result
};
use lmdb::Transaction;
use redis::Commands;

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

#[derive(Debug, Clone)]
pub struct DictionaryConfig {
  adapter:   DictionaryAdapter,
  lmdb_path: Option<PathBuf>,
  redis_url: Option<String>,
  namespace: Option<String>
}

impl DictionaryConfig {
  pub fn new(
    adapter: DictionaryAdapter
  ) -> Self {
    Self {
      adapter,
      lmdb_path: None,
      redis_url: std::env::var(
        "CITE_OTTER_REDIS_URL"
      )
      .ok()
      .or_else(|| {
        std::env::var("REDIS_URL").ok()
      }),
      namespace: None
    }
  }

  pub fn with_lmdb_path(
    mut self,
    path: impl Into<PathBuf>
  ) -> Self {
    self.lmdb_path = Some(path.into());
    self
  }

  pub fn with_redis_url(
    mut self,
    url: impl Into<String>
  ) -> Self {
    self.redis_url = Some(url.into());
    self
  }

  pub fn with_namespace(
    mut self,
    namespace: impl Into<String>
  ) -> Self {
    self.namespace =
      Some(namespace.into());
    self
  }

  pub fn open(
    &self
  ) -> Result<Dictionary> {
    let backend = match self.adapter {
      | DictionaryAdapter::Memory => {
        DictionaryBackend::Memory
      }
      | DictionaryAdapter::Gdbm => {
        DictionaryBackend::Memory
      }
      | DictionaryAdapter::Lmdb => {
        let path = self
          .lmdb_path
          .as_ref()
          .cloned()
          .unwrap_or_else(|| {
            Path::new("target")
              .join("dictionaries")
              .join("lmdb")
          });
        DictionaryBackend::Lmdb(
          LmdbBackend::open(&path)?
        )
      }
      | DictionaryAdapter::Redis => {
        let url = self
          .redis_url
          .clone()
          .context(
            "redis adapter requires \
             CITE_OTTER_REDIS_URL or \
             REDIS_URL"
          )?;
        let namespace = self
          .namespace
          .clone()
          .unwrap_or_else(|| {
            "cite-otter:place".into()
          });
        DictionaryBackend::Redis(
          RedisBackend::open(
            &url, namespace
          )?
        )
      }
    };

    Ok(Dictionary {
      adapter: self.adapter,
      backend
    })
  }

  pub fn open_or_memory(
    &self
  ) -> Dictionary {
    self.open().unwrap_or_else(|_| {
      Dictionary {
        adapter: self.adapter,
        backend:
          DictionaryBackend::Memory
      }
    })
  }
}

#[derive(Debug)]
pub struct Dictionary {
  adapter: DictionaryAdapter,
  backend: DictionaryBackend
}

impl Dictionary {
  pub fn create(
    adapter: DictionaryAdapter
  ) -> Self {
    DictionaryConfig::new(adapter)
      .open_or_memory()
  }

  pub fn try_create(
    config: DictionaryConfig
  ) -> Result<Self> {
    config.open()
  }

  pub fn open(self) -> Self {
    self
  }

  pub fn lookup(
    &self,
    term: &str
  ) -> Vec<DictionaryCode> {
    let terms = normalized_terms(term);
    if self
      .backend
      .contains_place(&terms)
    {
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

#[derive(Debug)]
enum DictionaryBackend {
  Memory,
  Lmdb(LmdbBackend),
  Redis(RedisBackend)
}

impl DictionaryBackend {
  fn contains_place(
    &self,
    terms: &[String]
  ) -> bool {
    match self {
      | Self::Memory => {
        terms.iter().any(|term| {
          PLACE_NAMES
            .iter()
            .any(|&name| term == name)
        })
      }
      | Self::Lmdb(backend) => {
        backend.contains_any(terms)
      }
      | Self::Redis(backend) => {
        backend.contains_any(terms)
      }
    }
  }
}

#[derive(Debug)]
struct LmdbBackend {
  env: lmdb::Environment,
  db:  lmdb::Database
}

impl LmdbBackend {
  fn open(path: &Path) -> Result<Self> {
    std::fs::create_dir_all(path)?;
    let env = lmdb::Environment::new()
      .set_max_dbs(1)
      .set_map_size(10 * 1024 * 1024)
      .open(path)
      .context(
        "open lmdb environment"
      )?;
    let db = env
      .create_db(
        Some("places"),
        lmdb::DatabaseFlags::empty()
      )
      .context("create lmdb db")?;
    let backend = Self {
      env,
      db
    };
    backend.seed_places()?;
    Ok(backend)
  }

  fn seed_places(&self) -> Result<()> {
    let mut txn =
      self.env.begin_rw_txn()?;
    for place in PLACE_NAMES {
      let _ = txn.put(
        self.db,
        place,
        b"1",
        lmdb::WriteFlags::NO_OVERWRITE
      );
    }
    txn.commit()?;
    Ok(())
  }

  fn contains_any(
    &self,
    terms: &[String]
  ) -> bool {
    let txn =
      match self.env.begin_ro_txn() {
        | Ok(txn) => txn,
        | Err(_) => return false
      };
    for term in terms {
      if txn.get(self.db, term).is_ok()
      {
        return true;
      }
    }
    false
  }
}

#[derive(Debug)]
struct RedisBackend {
  client:    redis::Client,
  namespace: String
}

impl RedisBackend {
  fn open(
    url: &str,
    namespace: String
  ) -> Result<Self> {
    let client =
      redis::Client::open(url)
        .context("open redis client")?;
    let backend = Self {
      client,
      namespace
    };
    backend.seed_places()?;
    Ok(backend)
  }

  fn seed_places(&self) -> Result<()> {
    let mut conn =
      self.client.get_connection()?;
    for place in PLACE_NAMES {
      let key = self.key(place);
      let _: () = redis::cmd("SETNX")
        .arg(&key)
        .arg("1")
        .query(&mut conn)?;
    }
    Ok(())
  }

  fn contains_any(
    &self,
    terms: &[String]
  ) -> bool {
    let mut conn = match self
      .client
      .get_connection()
    {
      | Ok(conn) => conn,
      | Err(_) => return false
    };
    for term in terms {
      let key = self.key(term);
      if let Ok(true) =
        conn.exists(&key)
      {
        return true;
      }
    }
    false
  }

  fn key(
    &self,
    term: &str
  ) -> String {
    format!(
      "{}:{}",
      self.namespace, term
    )
  }
}

fn normalized_terms(
  term: &str
) -> Vec<String> {
  let normalized = term
    .to_lowercase()
    .chars()
    .map(|c| {
      if c.is_ascii_alphanumeric() {
        c
      } else {
        ' '
      }
    })
    .collect::<String>();

  normalized
    .split_whitespace()
    .filter(|item| !item.is_empty())
    .map(|item| item.to_string())
    .collect()
}

static PLACE_NAMES: &[&str] =
  &["philippines", "italy"];
