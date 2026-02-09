use std::collections::HashMap;
use std::path::{
  Path,
  PathBuf
};

use anyhow::{
  Context,
  Result
};
#[cfg(not(feature = "gdbm"))]
use anyhow::anyhow;
#[cfg(feature = "gdbm")]
use gnudbm::{
  Error as GdbmError,
  GdbmOpener
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
  Name,
  Place,
  Publisher,
  Journal
}

impl DictionaryCode {
  pub fn bit(self) -> u32 {
    match self {
      | DictionaryCode::Name => 1,
      | DictionaryCode::Place => 2,
      | DictionaryCode::Publisher => 4,
      | DictionaryCode::Journal => 8
    }
  }

  pub fn from_tag(
    tag: &str
  ) -> Option<Self> {
    match tag
      .trim()
      .to_ascii_lowercase()
      .as_str()
    {
      | "name" => {
        Some(DictionaryCode::Name)
      }
      | "place" => {
        Some(DictionaryCode::Place)
      }
      | "publisher" => {
        Some(DictionaryCode::Publisher)
      }
      | "journal" => {
        Some(DictionaryCode::Journal)
      }
      | _ => None
    }
  }

  pub fn from_value(
    value: u32
  ) -> Vec<Self> {
    let mut codes = Vec::new();
    for code in &[
      DictionaryCode::Name,
      DictionaryCode::Place,
      DictionaryCode::Publisher,
      DictionaryCode::Journal
    ] {
      if value & code.bit() != 0 {
        codes.push(*code);
      }
    }
    codes
  }
}

#[derive(Debug, Clone, Copy)]
struct DictionaryValue(u32);

impl DictionaryValue {
  fn from_bytes(
    bytes: &[u8]
  ) -> Option<Self> {
    if bytes.len() == 4 {
      let mut buf = [0u8; 4];
      buf.copy_from_slice(bytes);
      return Some(Self(
        u32::from_le_bytes(buf)
      ));
    }
    let parsed =
      std::str::from_utf8(bytes)
        .ok()?
        .trim()
        .parse::<u32>()
        .ok()?;
    Some(Self(parsed))
  }

  fn from_string(
    value: &str
  ) -> Option<Self> {
    let parsed = value
      .trim()
      .parse::<u32>()
      .ok()?;
    Some(Self(parsed))
  }

  fn bytes(self) -> [u8; 4] {
    self.0.to_le_bytes()
  }
}

#[derive(Debug, Clone)]
pub struct DictionaryConfig {
  adapter:   DictionaryAdapter,
  lmdb_path: Option<PathBuf>,
  gdbm_path: Option<PathBuf>,
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
      gdbm_path: None,
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

  pub fn with_gdbm_path(
    mut self,
    path: impl Into<PathBuf>
  ) -> Self {
    self.gdbm_path = Some(path.into());
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
        DictionaryBackend::Memory(
          MemoryBackend::new()
        )
      }
      | DictionaryAdapter::Gdbm => {
        let path = resolve_backend_path(
          self.gdbm_path.as_ref(),
          "gdbm",
          "places.db"
        );
        DictionaryBackend::Gdbm(
          GdbmBackend::open(&path)?
        )
      }
      | DictionaryAdapter::Lmdb => {
        let path = resolve_backend_path(
          self.lmdb_path.as_ref(),
          "lmdb",
          ""
        );
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
          DictionaryBackend::Memory(
            MemoryBackend::new()
          )
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
    let mut value = 0u32;
    for term in terms {
      if let Some(found) =
        self.backend.get_value(&term)
      {
        value |= found;
      }
    }
    DictionaryCode::from_value(value)
  }

  pub fn adapter(
    &self
  ) -> DictionaryAdapter {
    self.adapter
  }

  pub fn import_terms(
    &mut self,
    code: DictionaryCode,
    terms: impl IntoIterator<Item = String>
  ) -> Result<usize> {
    let entries = terms
      .into_iter()
      .map(|term| (term, code.bit()));
    self.import_entries(entries)
  }

  pub fn import_entries(
    &mut self,
    entries: impl IntoIterator<
      Item = (String, u32)
    >
  ) -> Result<usize> {
    let mut prepared =
      HashMap::<String, u32>::new();
    for (term, value) in entries {
      let term = term.trim();
      if term.is_empty() || value == 0 {
        continue;
      }
      for token in
        normalized_terms(term)
      {
        let entry = prepared
          .entry(token)
          .or_insert(0);
        *entry |= value;
      }
    }
    let prepared = prepared
      .into_iter()
      .collect::<Vec<_>>();
    self
      .backend
      .merge_entries(&prepared)
  }
}

#[derive(Debug)]
enum DictionaryBackend {
  Memory(MemoryBackend),
  Gdbm(GdbmBackend),
  Lmdb(LmdbBackend),
  Redis(RedisBackend)
}

impl DictionaryBackend {
  fn get_value(
    &self,
    term: &str
  ) -> Option<u32> {
    match self {
      | Self::Memory(backend) => {
        backend.get_value(term)
      }
      | Self::Gdbm(backend) => {
        backend.get_value(term)
      }
      | Self::Lmdb(backend) => {
        backend.get_value(term)
      }
      | Self::Redis(backend) => {
        backend.get_value(term)
      }
    }
  }

  fn merge_entries(
    &mut self,
    entries: &[(String, u32)]
  ) -> Result<usize> {
    match self {
      | Self::Memory(backend) => {
        Ok(
          backend
            .merge_entries(entries)
        )
      }
      | Self::Gdbm(backend) => {
        backend.merge_entries(entries)
      }
      | Self::Lmdb(backend) => {
        backend.merge_entries(entries)
      }
      | Self::Redis(backend) => {
        backend.merge_entries(entries)
      }
    }
  }
}

#[derive(Debug)]
struct MemoryBackend {
  entries: HashMap<String, u32>
}

impl MemoryBackend {
  fn new() -> Self {
    let mut entries = HashMap::new();
    for place in PLACE_NAMES {
      entries.insert(
        place.to_string(),
        DictionaryCode::Place.bit()
      );
    }
    Self {
      entries
    }
  }

  fn get_value(
    &self,
    term: &str
  ) -> Option<u32> {
    self.entries.get(term).copied()
  }

  fn merge_entries(
    &mut self,
    entries: &[(String, u32)]
  ) -> usize {
    let mut updated = 0usize;
    for (term, value) in entries {
      let entry = self
        .entries
        .entry(term.clone())
        .or_insert(0);
      let next = *entry | *value;
      if next != *entry {
        *entry = next;
        updated += 1;
      }
    }
    updated
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
      let value = DictionaryValue(
        DictionaryCode::Place.bit()
      )
      .bytes();
      let _ = txn.put(
        self.db,
        place,
        &value,
        lmdb::WriteFlags::NO_OVERWRITE
      );
    }
    txn.commit()?;
    Ok(())
  }

  fn get_value(
    &self,
    term: &str
  ) -> Option<u32> {
    let txn =
      self.env.begin_ro_txn().ok()?;
    let key = term.to_string();
    let bytes =
      txn.get(self.db, &key).ok()?;
    DictionaryValue::from_bytes(bytes)
      .map(|v| v.0)
  }

  fn merge_entries(
    &mut self,
    entries: &[(String, u32)]
  ) -> Result<usize> {
    let mut txn =
      self.env.begin_rw_txn()?;
    let mut inserted = 0usize;
    for (term, value) in entries {
      let existing = txn
        .get(self.db, term)
        .ok()
        .and_then(|bytes| {
          DictionaryValue::from_bytes(
            bytes
          )
          .map(|v| v.0)
        })
        .unwrap_or(0);
      let merged = existing | *value;
      if merged != existing {
        let encoded =
          DictionaryValue(merged)
            .bytes();
        txn.put(
          self.db,
          term,
          &encoded,
          lmdb::WriteFlags::empty()
        )?;
        inserted += 1;
      }
    }
    txn.commit()?;
    Ok(inserted)
  }
}

#[cfg(feature = "gdbm")]
#[derive(Debug)]
struct GdbmBackend {
  handle: gnudbm::RwHandle
}

#[cfg(feature = "gdbm")]
impl GdbmBackend {
  fn open(path: &Path) -> Result<Self> {
    if let Some(parent) = path.parent()
    {
      std::fs::create_dir_all(parent)?;
    }
    let handle = GdbmOpener::new()
      .create(true)
      .readwrite(path)
      .context("open gdbm database")?;
    let mut backend = Self {
      handle
    };
    backend.seed_places()?;
    Ok(backend)
  }

  fn seed_places(
    &mut self
  ) -> Result<()> {
    for place in PLACE_NAMES {
      let value = DictionaryValue(
        DictionaryCode::Place.bit()
      )
      .bytes();
      let _ = self
        .handle
        .store(place, &value);
    }
    Ok(())
  }

  fn get_value(
    &self,
    term: &str
  ) -> Option<u32> {
    match self.handle.fetch(term) {
      | Ok(bytes) => {
        DictionaryValue::from_bytes(
          bytes.as_bytes()
        )
        .map(|v| v.0)
      }
      | Err(GdbmError::NoRecord) => {
        None
      }
      | Err(_) => None
    }
  }

  fn merge_entries(
    &mut self,
    entries: &[(String, u32)]
  ) -> Result<usize> {
    let mut updated = 0usize;
    for (term, value) in entries {
      let existing = match self
        .handle
        .fetch(term)
      {
        | Ok(bytes) => {
          DictionaryValue::from_bytes(
            bytes.as_bytes()
          )
          .map(|v| v.0)
          .unwrap_or(0)
        }
        | Err(GdbmError::NoRecord) => 0,
        | Err(_) => 0
      };
      let merged = existing | *value;
      if merged != existing {
        let encoded =
          DictionaryValue(merged)
            .bytes();
        let _ = self
          .handle
          .store(term, &encoded);
        updated += 1;
      }
    }
    Ok(updated)
  }
}

#[cfg(not(feature = "gdbm"))]
#[derive(Debug)]
struct GdbmBackend;

#[cfg(not(feature = "gdbm"))]
impl GdbmBackend {
  fn open(
    _path: &Path
  ) -> Result<Self> {
    Err(anyhow!(
      "gdbm support not enabled; \
       recompile with --features gdbm"
    ))
  }

  fn get_value(
    &self,
    _term: &str
  ) -> Option<u32> {
    None
  }

  fn merge_entries(
    &mut self,
    _entries: &[(String, u32)]
  ) -> Result<usize> {
    Err(anyhow!(
      "gdbm support not enabled; \
       recompile with --features gdbm"
    ))
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
      let value =
        DictionaryCode::Place.bit();
      let _: () = redis::cmd("SETNX")
        .arg(&key)
        .arg(value.to_string())
        .query(&mut conn)?;
    }
    Ok(())
  }

  fn get_value(
    &self,
    term: &str
  ) -> Option<u32> {
    let mut conn = self
      .client
      .get_connection()
      .ok()?;
    let key = self.key(term);
    let value: Option<String> =
      conn.get(&key).ok()?;
    DictionaryValue::from_string(
      value.as_deref().unwrap_or("")
    )
    .map(|v| v.0)
  }

  fn merge_entries(
    &mut self,
    entries: &[(String, u32)]
  ) -> Result<usize> {
    let mut conn =
      self.client.get_connection()?;
    let mut inserted = 0usize;
    for (term, value) in entries {
      let key = self.key(term);
      let existing: Option<String> =
        conn.get(&key)?;
      let existing_value = existing
        .as_deref()
        .and_then(
          DictionaryValue::from_string
        )
        .map(|v| v.0)
        .unwrap_or(0);
      let merged =
        existing_value | *value;
      if merged != existing_value {
        let _: () = redis::cmd("SET")
          .arg(&key)
          .arg(merged.to_string())
          .query(&mut conn)?;
        inserted += 1;
      }
    }
    Ok(inserted)
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

fn resolve_backend_path(
  candidate: Option<&PathBuf>,
  default_dir: &str,
  default_file: &str
) -> PathBuf {
  let base = candidate
    .cloned()
    .unwrap_or_else(|| {
      Path::new("target")
        .join("dictionaries")
        .join(default_dir)
    });
  if base.extension().is_some()
    || default_file.is_empty()
  {
    base
  } else {
    base.join(default_file)
  }
}

static PLACE_NAMES: &[&str] =
  &["philippines", "italy"];
