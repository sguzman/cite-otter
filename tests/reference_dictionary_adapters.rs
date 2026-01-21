use cite_otter::dictionary::{
  Dictionary,
  DictionaryAdapter
};

#[test]
#[ignore = "pending adapter \
            implementations"]
#[cfg(feature = "redis")]
fn redis_adapter_exists() {
  let dict = Dictionary::create(
    DictionaryAdapter::Redis
  );
  let _ = dict.lookup("redis");
}

#[test]
#[ignore = "pending adapter \
            implementations"]
#[cfg(feature = "lmdb")]
fn lmdb_adapter_exists() {
  let dict = Dictionary::create(
    DictionaryAdapter::Lmdb
  );
  let _ = dict.lookup("lmdb");
}

#[test]
#[ignore = "pending adapter \
            implementations"]
#[cfg(feature = "gdbm")]
fn gdbm_adapter_exists() {
  let dict = Dictionary::create(
    DictionaryAdapter::Gdbm
  );
  let _ = dict.lookup("gdbm");
}
