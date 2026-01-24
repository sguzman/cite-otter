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
