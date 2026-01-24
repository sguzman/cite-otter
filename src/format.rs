use clap::ValueEnum;
use serde_json::{
  Map,
  Value
};

use crate::normalizer::NormalizationConfig;
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
pub struct Format {
  normalization: NormalizationConfig
}

impl Default for Format {
  fn default() -> Self {
    Self::new()
  }
}

impl Format {
  pub fn new() -> Self {
    Self {
      normalization:
        NormalizationConfig::default()
    }
  }

  pub fn with_normalization(
    normalization: NormalizationConfig
  ) -> Self {
    Self {
      normalization
    }
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
        self
          .normalization
          .apply_to_map(&mut map);
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
        self
          .normalization
          .apply_to_map(&mut map);
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
  if !map.contains_key("address") {
    rename_field(
      map,
      "publisher-place",
      "address"
    );
  }

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

  if let Some(date_parts) =
    extract_date_parts(&map)
  {
    record.insert(
      "issued".into(),
      Value::Object(
        Map::from_iter([(
          "date-parts".into(),
          Value::Array(vec![Value::Array(
            date_parts
              .into_iter()
              .map(|part| {
                Value::Number(
                  serde_json::Number::from(
                    part
                  )
                )
              })
              .collect()
          )])
        )])
      )
    );
  }

  if let Some(pages) =
    extract_first_value_from_map(
      &map, "pages"
    )
  {
    if !pages.is_empty() {
      record.insert(
        "page".into(),
        Value::String(pages)
      );
    }
  }

  for key in
    ["author", "editor", "translator"]
  {
    if let Some(values) =
      extract_values_from_map(&map, key)
    {
      record.insert(
        key.into(),
        Value::Array(
          values
            .into_iter()
            .map(Value::String)
            .collect()
        )
      );
    }
  }

  for key in [
    "container-title",
    "collection-title",
    "collection-number",
    "journal",
    "publisher",
    "publisher-place",
    "address",
    "volume",
    "issue",
    "doi",
    "url",
    "isbn",
    "issn"
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

fn extract_values_from_map(
  map: &Map<String, Value>,
  key: &str
) -> Option<Vec<String>> {
  map
    .get(key)
    .and_then(|value| {
      value.as_array().map(|items| {
        items
          .iter()
          .filter_map(Value::as_str)
          .map(|value| {
            value.to_string()
          })
          .collect::<Vec<_>>()
      })
    })
    .filter(|values| !values.is_empty())
}

fn extract_date_parts(
  map: &Map<String, Value>
) -> Option<Vec<i32>> {
  let value =
    extract_first_value_from_map(
      map, "date"
    )?;
  let parts = value
    .split(|c: char| {
      !c.is_ascii_digit()
    })
    .filter(|part| !part.is_empty())
    .filter_map(|part| {
      part.parse().ok()
    })
    .collect::<Vec<i32>>();
  if parts.is_empty() {
    None
  } else {
    Some(parts)
  }
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
