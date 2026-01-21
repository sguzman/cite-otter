use cite_otter::dictionary::{
  Dictionary,
  DictionaryAdapter,
  DictionaryCode
};

#[test]
#[ignore = "pending dictionary \
            implementation"]
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
