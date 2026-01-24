use std::env;

use cite_otter::dictionary::{
  Dictionary,
  DictionaryAdapter,
  DictionaryCode,
  DictionaryConfig
};

#[test]
fn lmdb_backend_lookup_reads_seeded_data()
 {
  let temp_dir = tempfile::tempdir()
    .expect("lmdb tempdir");
  let config = DictionaryConfig::new(
    DictionaryAdapter::Lmdb
  )
  .with_lmdb_path(temp_dir.path());
  let dictionary =
    Dictionary::try_create(config)
      .expect("lmdb dictionary opens");

  let codes =
    dictionary.lookup("Italy");
  assert_eq!(
    codes,
    vec![DictionaryCode::Place],
    "lmdb adapter should resolve \
     place names"
  );
}

#[test]
fn lmdb_backend_imports_terms() {
  let temp_dir = tempfile::tempdir()
    .expect("lmdb tempdir");
  let mut dictionary =
    Dictionary::try_create(
      DictionaryConfig::new(
        DictionaryAdapter::Lmdb
      )
      .with_lmdb_path(temp_dir.path())
    )
    .expect("lmdb dictionary opens");

  dictionary
    .import_terms(
      DictionaryCode::Place,
      vec!["Wakanda".to_string()]
    )
    .expect("lmdb import");

  let codes =
    dictionary.lookup("Wakanda");
  assert_eq!(
    codes,
    vec![DictionaryCode::Place],
    "lmdb adapter should import place \
     names"
  );
}

#[test]
fn redis_backend_lookup_reads_seeded_data()
 {
  let redis_url =
    env::var("CITE_OTTER_REDIS_URL")
      .or_else(|_| {
        env::var("REDIS_URL")
      })
      .ok();
  let Some(redis_url) = redis_url
  else {
    return;
  };

  let config = DictionaryConfig::new(
    DictionaryAdapter::Redis
  )
  .with_redis_url(redis_url)
  .with_namespace(
    "cite-otter-test".to_string()
  );
  let dictionary =
    Dictionary::try_create(config)
      .expect("redis dictionary opens");

  let codes =
    dictionary.lookup("Italy");
  assert_eq!(
    codes,
    vec![DictionaryCode::Place],
    "redis adapter should resolve \
     place names"
  );
}

#[test]
fn redis_backend_imports_terms() {
  let redis_url =
    env::var("CITE_OTTER_REDIS_URL")
      .or_else(|_| {
        env::var("REDIS_URL")
      })
      .ok();
  let Some(redis_url) = redis_url
  else {
    return;
  };

  let mut dictionary =
    Dictionary::try_create(
      DictionaryConfig::new(
        DictionaryAdapter::Redis
      )
      .with_redis_url(redis_url)
      .with_namespace(
        "cite-otter-test".to_string()
      )
    )
    .expect("redis dictionary opens");

  dictionary
    .import_terms(
      DictionaryCode::Place,
      vec!["Wakanda".to_string()]
    )
    .expect("redis import");

  let codes =
    dictionary.lookup("Wakanda");
  assert_eq!(
    codes,
    vec![DictionaryCode::Place],
    "redis adapter should import \
     place names"
  );
}

#[cfg(feature = "gdbm")]
#[test]
fn gdbm_backend_lookup_reads_seeded_data()
 {
  let temp_dir = tempfile::tempdir()
    .expect("gdbm tempdir");
  let db_path =
    temp_dir.path().join("places.db");
  let config = DictionaryConfig::new(
    DictionaryAdapter::Gdbm
  )
  .with_gdbm_path(db_path);
  let dictionary =
    Dictionary::try_create(config)
      .expect("gdbm dictionary opens");

  let codes =
    dictionary.lookup("Italy");
  assert_eq!(
    codes,
    vec![DictionaryCode::Place],
    "gdbm adapter should resolve \
     place names"
  );
}

#[cfg(feature = "gdbm")]
#[test]
fn gdbm_backend_imports_terms() {
  let temp_dir = tempfile::tempdir()
    .expect("gdbm tempdir");
  let db_path =
    temp_dir.path().join("places.db");
  let mut dictionary =
    Dictionary::try_create(
      DictionaryConfig::new(
        DictionaryAdapter::Gdbm
      )
      .with_gdbm_path(db_path)
    )
    .expect("gdbm dictionary opens");

  dictionary
    .import_terms(
      DictionaryCode::Place,
      vec!["Wakanda".to_string()]
    )
    .expect("gdbm import");

  let codes =
    dictionary.lookup("Wakanda");
  assert_eq!(
    codes,
    vec![DictionaryCode::Place],
    "gdbm adapter should import place \
     names"
  );
}
