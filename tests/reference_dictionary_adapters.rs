#![allow(dead_code)]
use cite_otter::dictionary::{
  Dictionary,
  DictionaryAdapter
};

#[test]
fn redis_adapter_exists() {
  let dict = Dictionary::create(
    DictionaryAdapter::Redis
  );
  let _ = dict.lookup("redis");
}

#[test]
fn lmdb_adapter_exists() {
  let dict = Dictionary::create(
    DictionaryAdapter::Lmdb
  );
  let _ = dict.lookup("lmdb");
}

#[test]
fn gdbm_adapter_exists() {
  let dict = Dictionary::create(
    DictionaryAdapter::Gdbm
  );
  let _ = dict.lookup("gdbm");
}
