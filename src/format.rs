use clap::ValueEnum;
use serde_json::{
  Map,
  Value
};

use crate::normalizer::journal::Normalizer as JournalNormalizer;
use crate::parser::{
  FieldValue,
  Reference
};

#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  ValueEnum,
)]
pub enum ParseFormat {
  Json,
  BibTeX,
  Csl
}

#[derive(Debug, Clone)]
pub struct Format;

impl Default for Format {
  fn default() -> Self {
    Self::new()
  }
}

impl Format {
  pub fn new() -> Self {
    Self
  }

  pub fn to_bibtex(
    &self,
    references: &[Reference]
  ) -> String {
    references
      .iter()
      .enumerate()
      .map(|(idx, reference)| {
        let mut map =
          reference_to_map(reference);
        JournalNormalizer::new()
          .normalize(&mut map);
        normalize_bibtex_entry(
          &mut map
        );
        let entry_type =
          entry_type_for(&mut map);
        fields_to_bibtex(
          idx, entry_type, &map
        )
      })
      .collect::<Vec<_>>()
      .join("\n\n")
  }

  pub fn to_json(
    &self,
    references: &[Reference]
  ) -> String {
    serde_json::to_string_pretty(
      references
    )
    .unwrap_or_else(|_| "[]".into())
  }

  pub fn to_csl(
    &self,
    references: &[Reference]
  ) -> String {
    references
      .iter()
      .enumerate()
      .map(|(idx, reference)| {
        let mut map =
          reference_to_map(reference);
        JournalNormalizer::new()
          .normalize(&mut map);
        csl_entry(idx, map)
      })
      .collect::<Vec<_>>()
      .join("\n")
  }

  pub fn to_value(
    &self,
    references: &[Reference]
  ) -> Value {
    serde_json::to_value(references)
      .unwrap_or(Value::Null)
  }
}

fn reference_to_map(
  reference: &Reference
) -> Map<String, Value> {
  reference
    .fields()
    .iter()
    .filter_map(|(key, value)| {
      let entries =
        field_value_strings(value);
      if entries.is_empty() {
        None
      } else {
        Some((
          key.clone(),
          Value::Array(
            entries
              .into_iter()
              .map(Value::String)
              .collect()
          )
        ))
      }
    })
    .collect()
}

fn field_value_strings(
  value: &FieldValue
) -> Vec<String> {
  match value {
    | FieldValue::Single(text) => {
      vec![text.clone()]
    }
    | FieldValue::List(items) => {
      items.clone()
    }
    | FieldValue::Authors(authors) => {
      authors
        .iter()
        .map(|author| {
          if author.given.is_empty() {
            author.family.clone()
          } else {
            format!(
              "{}, {}",
              author.family,
              author.given
            )
          }
        })
        .collect()
    }
  }
}

fn normalize_bibtex_entry(
  map: &mut Map<String, Value>
) {
  if let Some(value) =
    map.remove("type")
  {
    map.insert("type".into(), value);
  }

  rename_field(
    map,
    "container-title",
    "booktitle"
  );
  rename_field(
    map,
    "collection-title",
    "series"
  );
  rename_field(
    map, "location", "address"
  );

  if let Some(value) =
    map.remove("volume")
  {
    map.insert("volume".into(), value);
  }
  if let Some(value) =
    map.remove("issue")
  {
    map.insert("issue".into(), value);
  }
}

fn entry_type_for(
  map: &mut Map<String, Value>
) -> String {
  let entry_type =
    extract_first_value(map, "type")
      .unwrap_or_else(|| "misc".into());
  match entry_type.as_str() {
    | "article" => {
      rename_field(
        map,
        "booktitle",
        "journal"
      );
      rename_field(
        map, "issue", "number"
      );
      entry_type
    }
    | "techreport" => {
      rename_field(
        map,
        "publisher",
        "institution"
      );
      entry_type
    }
    | "thesis" => {
      rename_field(
        map,
        "publisher",
        "school"
      );
      entry_type
    }
    | _ => entry_type
  }
}

fn fields_to_bibtex(
  idx: usize,
  entry_type: String,
  map: &Map<String, Value>
) -> String {
  let fields = map
    .iter()
    .filter_map(|(key, value)| {
      let content = value
        .as_array()
        .and_then(|items| {
          items
            .first()
            .and_then(Value::as_str)
        });
      content.map(|value| {
        format!("  {key} = {{{value}}}")
      })
    })
    .collect::<Vec<_>>()
    .join("\n");

  format!(
    "@{entry_type}{{citeotter{idx},\\
     n{fields}\n}}"
  )
}

fn csl_entry(
  idx: usize,
  map: Map<String, Value>
) -> String {
  let mut record = Map::new();
  record.insert(
    "id".into(),
    Value::String(format!(
      "citeotter{idx}"
    ))
  );

  if let Some(title) =
    extract_first_value_from_map(
      &map, "title"
    )
  {
    record.insert(
      "title".into(),
      Value::String(title)
    );
  }

  for key in [
    "container-title",
    "collection-title",
    "journal",
    "publisher",
    "address",
    "doi",
    "url"
  ] {
    if let Some(value) =
      extract_first_value_from_map(
        &map, key
      )
    {
      record.insert(
        key.into(),
        Value::String(value)
      );
    }
  }

  serde_json::to_string(&Value::Object(
    record
  ))
  .unwrap_or_else(|_| "{}".into())
}

fn extract_first_value(
  map: &mut Map<String, Value>,
  key: &str
) -> Option<String> {
  map.remove(key).and_then(|value| {
    value.as_array().and_then(|items| {
      items
        .first()
        .and_then(Value::as_str)
        .map(|s| s.to_string())
    })
  })
}

fn extract_first_value_from_map(
  map: &Map<String, Value>,
  key: &str
) -> Option<String> {
  map.get(key).and_then(|value| {
    value.as_array().and_then(|items| {
      items
        .first()
        .and_then(Value::as_str)
        .map(|s| s.to_string())
    })
  })
}

fn rename_field(
  map: &mut Map<String, Value>,
  from: &str,
  to: &str
) {
  if let Some(value) = map.remove(from)
  {
    map
      .entry(to.to_string())
      .or_insert(value);
  }
}
