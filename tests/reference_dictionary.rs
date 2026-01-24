use cite_otter::dictionary::{
  Dictionary,
  DictionaryAdapter,
  DictionaryCode
};

#[test]
fn place_names_are_tagged() {
  let dict = Dictionary::create(
    DictionaryAdapter::Memory
  )
  .open();

  for place in &["philippines", "italy"]
  {
    let codes = dict.lookup(place);
    assert!(
      codes.contains(
        &DictionaryCode::Place
      ),
      "expected {place} to map to \
       Place"
    );
  }
}

#[test]
fn dictionary_imports_merge_codes() {
  let mut dict = Dictionary::create(
    DictionaryAdapter::Memory
  )
  .open();

  dict
    .import_entries(vec![
      (
        "Nature".to_string(),
        DictionaryCode::Journal.bit()
      ),
      (
        "Nature".to_string(),
        DictionaryCode::Publisher.bit()
      ),
    ])
    .expect("dictionary import");

  let codes = dict.lookup("Nature");
  assert!(
    codes.contains(
      &DictionaryCode::Journal
    ),
    "expected Nature to map to Journal"
  );
  assert!(
    codes.contains(
      &DictionaryCode::Publisher
    ),
    "expected Nature to map to \
     Publisher"
  );
}
