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
    let mut key_counts =
      std::collections::HashMap::new();
    let mut output = references
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
        let key =
          bibtex_key_for(&map, idx, &mut key_counts);
        fields_to_bibtex(
          &key, entry_type, &map
        )
      })
      .collect::<Vec<_>>()
      .join("\n");
    if !output.ends_with('\n') {
      output.push('\n');
    }
    output
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
      if text.trim().is_empty() {
        Vec::new()
      } else {
        vec![text.clone()]
      }
    }
    | FieldValue::List(items) => {
      items
        .iter()
        .filter(|item| !item.trim().is_empty())
        .cloned()
        .collect()
    }
    | FieldValue::Authors(authors) => {
      authors
        .iter()
        .map(|author| {
          if author.given.is_empty() {
            author.family.clone()
          } else {
            let given =
              normalize_given_initials(
                &author.given
              );
            format!(
              "{}, {}",
              author.family,
              given
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

  normalize_bibtex_date(map);
  map.remove("language");
  map.remove("scripts");

  if let Some(value) =
    map.remove("isbn")
  {
    map.insert("isbn".into(), value);
  }
  if let Some(value) =
    map.remove("issn")
  {
    map.insert("issn".into(), value);
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
  } else {
    map.remove("publisher-place");
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
  let raw_type =
    extract_first_value(map, "type")
      .unwrap_or_else(|| "misc".into());
  let entry_type = match raw_type.as_str() {
    | "article-journal" => "article",
    | "chapter" => "incollection",
    | "manuscript" => "unpublished",
    | "report" => "techreport",
    | "paper-conference" => {
      "inproceedings"
    }
    | _ => raw_type.as_str()
  }
  .to_string();
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
  key: &str,
  entry_type: String,
  map: &Map<String, Value>
) -> String {
  let fields = bibtex_fields(map);
  let total = fields.len();
  let mut rendered = Vec::with_capacity(total);
  for (idx, (key, value)) in
    fields.into_iter().enumerate()
  {
    let suffix =
      if idx + 1 == total {
        ""
      } else {
        ","
      };
    rendered.push(format!(
      "  {key} = {{{value}}}{suffix}"
    ));
  }
  let fields = rendered.join("\n");

  format!(
    "@{entry_type}{{{key},\n{fields}\n}}"
  )
}

fn csl_entry(
  idx: usize,
  map: Map<String, Value>
) -> String {
  let _ = idx;

  let mut entries: Vec<(String, Value)> =
    Vec::new();
  let mut push = |key: &str, value: Value| {
    entries.push((key.to_string(), value));
  };

  for key in
    ["author", "editor", "translator"]
  {
    if let Some(values) =
      extract_values_from_map(&map, key)
    {
      push(
        key,
        Value::Array(
          values
            .into_iter()
            .map(csl_name_value)
            .collect()
        )
      );
    }
  }

  if let Some(title) =
    extract_first_value_from_map(
      &map, "title"
    )
  {
    push(
      "title",
      Value::String(title)
    );
  }
  if let Some(value) =
    extract_first_value_from_map(
      &map,
      "citation-number"
    )
  {
    push(
      "citation-number",
      Value::String(value)
    );
  }

  for key in [
    "edition",
    "publisher",
    "note",
    "genre",
    "collection-title",
    "collection-number",
    "volume",
    "issue",
    "isbn",
    "issn"
  ] {
    if let Some(value) =
      extract_first_value_from_map(
        &map, key
      )
    {
      push(
        key,
        Value::String(
          sanitize_csl_value(&value)
        )
      );
    }
  }

  if let Some(value) =
    extract_first_value_from_map(
      &map,
      "container-title"
    )
    .or_else(|| {
      extract_first_value_from_map(
        &map, "journal"
      )
    })
  {
    push(
      "container-title",
      Value::String(
        sanitize_csl_value(&value)
      )
    );
  }

  if let Some(entry_type) =
    extract_first_value_from_map(
      &map, "type"
    )
  {
    push(
      "type",
      Value::String(entry_type)
    );
  }

  if let Some(issued) =
    extract_csl_issued(&map)
  {
    push(
      "issued",
      Value::String(issued)
    );
  }

  if let Some(pages) =
    extract_first_value_from_map(
      &map, "pages"
    )
  {
    if !pages.is_empty() {
      push(
        "page",
        Value::String(pages)
      );
    }
  }

  if let Some(value) =
    extract_first_value_from_map(
      &map,
      "publisher-place"
    )
    .or_else(|| {
      extract_first_value_from_map(
        &map, "address"
      )
    })
  {
    push(
      "publisher-place",
      Value::String(
        sanitize_csl_value(&value)
      )
    );
  }

  if let Some(value) =
    extract_first_value_from_map(&map, "url")
  {
    push(
      "URL",
      Value::String(
        sanitize_csl_value(&value)
      )
    );
  }
  if let Some(value) =
    extract_first_value_from_map(&map, "doi")
  {
    push(
      "DOI",
      Value::String(
        sanitize_csl_value(&value)
      )
    );
  }

  let mut output = String::from("{");
  for (idx, (key, value)) in
    entries.iter().enumerate()
  {
    let key_json =
      serde_json::to_string(key)
        .unwrap_or_else(|_| "\"\"".into());
    let value_json =
      serde_json::to_string(value)
        .unwrap_or_else(|_| "null".into());
    output.push_str(&key_json);
    output.push(':');
    output.push_str(&value_json);
    if idx + 1 < entries.len() {
      output.push(',');
    }
  }
  output.push('}');
  output
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

fn csl_name_value(
  name: String
) -> Value {
  let lower = name.to_lowercase();
  if lower.contains(" by ") {
    return Value::Object(
      Map::from_iter([(
        "literal".into(),
        Value::String(name)
      )])
    );
  }
  if let Some((family, given)) =
    split_name(&name)
  {
    let mut object = Map::new();
    object.insert(
      "family".into(),
      Value::String(family)
    );
    if !given.is_empty() {
      object.insert(
        "given".into(),
        Value::String(given)
      );
    }
    Value::Object(object)
  } else {
    Value::Object(Map::from_iter([(
      "literal".into(),
      Value::String(name)
    )]))
  }
}

fn split_name(
  name: &str
) -> Option<(String, String)> {
  let trimmed = name.trim();
  if trimmed.is_empty() {
    return None;
  }
  if let Some((family, given)) =
    trimmed.split_once(',')
  {
    return Some((
      family.trim().to_string(),
      given.trim().to_string()
    ));
  }
  let parts = trimmed
    .split_whitespace()
    .collect::<Vec<_>>();
  if parts.len() < 2 {
    return None;
  }
  let family = parts
    .last()
    .unwrap_or(&"")
    .to_string();
  let given =
    parts[..parts.len() - 1].join(" ");
  Some((family, given))
}

fn extract_csl_issued(
  map: &Map<String, Value>
) -> Option<String> {
  let array = map.get("date")?.as_array()?;
  let mut parts = array
    .iter()
    .filter_map(Value::as_str)
    .map(|value| value.trim())
    .filter(|value| !value.is_empty())
    .collect::<Vec<_>>();
  if parts.is_empty() {
    return None;
  }
  let issued = if parts.len() == 1 {
    parts.remove(0).to_string()
  } else if parts.len() == 2
    && parts
      .iter()
      .all(|value| value.len() == 4)
  {
    format!("{}/{}", parts[0], parts[1])
  } else if parts.len() >= 3 {
    format!("{}-{}-{}", parts[0], parts[1], parts[2])
  } else {
    parts.join("-")
  };
  if map.get("date-circa").is_some() {
    Some(format!("{issued}~"))
  } else {
    Some(issued)
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

fn bibtex_fields(
  map: &Map<String, Value>
) -> Vec<(String, String)> {
  let date_is_circa = map
    .get("date")
    .and_then(Value::as_array)
    .and_then(|items| items.first())
    .and_then(Value::as_str)
    .map(|value| value.trim_end().ends_with('~'))
    .unwrap_or(false);
  let mut entries = map
    .iter()
    .filter_map(|(key, value)| {
      let content =
        if is_bibtex_name_field(key) {
          value.as_array().map(|items| {
            items
              .iter()
              .filter_map(Value::as_str)
              .map(sanitize_bibtex_name_value)
              .collect::<Vec<_>>()
              .join(" and ")
          })
        } else {
          value
            .as_array()
            .and_then(|items| {
              items
                .first()
                .and_then(Value::as_str)
            })
            .map(sanitize_bibtex_value)
        };
      content.map(|value| (key.clone(), value))
    })
    .collect::<Vec<_>>();
  let order = [
    "author",
    "title",
    "edition",
    "booktitle",
    "journal",
    "series",
    "volume",
    "number",
    "publisher",
    "date",
    "institution",
    "school",
    "pages",
    "address",
    "doi",
    "url",
    "isbn",
    "issn",
    "note"
  ];
  entries.sort_by(|(left, _), (right, _)| {
    let left_idx = order
      .iter()
      .position(|key| key == left)
      .unwrap_or(usize::MAX);
    let right_idx = order
      .iter()
      .position(|key| key == right)
      .unwrap_or(usize::MAX);
    let left_idx = if left == "date" && date_is_circa {
      1
    } else {
      left_idx
    };
    let right_idx = if right == "date" && date_is_circa {
      1
    } else {
      right_idx
    };
    left_idx
      .cmp(&right_idx)
      .then_with(|| left.cmp(right))
  });
  entries
}

fn sanitize_bibtex_value(
  value: &str
) -> String {
  strip_terminal_punct(value)
    .replace('\n', " ")
    .replace('\r', " ")
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ")
}

fn sanitize_bibtex_name_value(
  value: &str
) -> String {
  value
    .replace('\n', " ")
    .replace('\r', " ")
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ")
}

fn sanitize_csl_value(
  value: &str
) -> String {
  strip_terminal_punct(value)
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ")
}

fn strip_terminal_punct(
  value: &str
) -> String {
  value
    .trim()
    .trim_end_matches(|c: char| {
      matches!(c, '.' | ',' | ';')
    })
    .to_string()
}

fn normalize_given_initials(
  value: &str
) -> String {
  value
    .split_whitespace()
    .map(|part| {
      if part.len() == 1
        && part.chars().all(|c| c.is_alphabetic())
      {
        format!("{part}.")
      } else {
        part.to_string()
      }
    })
    .collect::<Vec<_>>()
    .join(" ")
}

fn is_bibtex_name_field(
  key: &str
) -> bool {
  matches!(
    key,
    "author" | "editor" | "translator"
  )
}

fn normalize_bibtex_date(
  map: &mut Map<String, Value>
) {
  let has_circa =
    map.get("date-circa").is_some();
  let Some(value) = map.get_mut("date")
  else {
    return;
  };
  let Some(items) = value.as_array()
  else {
    return;
  };
  let mut parts = items
    .iter()
    .filter_map(Value::as_str)
    .map(|value| value.trim())
    .filter(|value| !value.is_empty())
    .collect::<Vec<_>>();
  if parts.is_empty() {
    return;
  }
  let mut date = if parts.len() == 1 {
    parts.remove(0).to_string()
  } else if parts.len() == 2
    && parts
      .iter()
      .all(|value| value.len() == 4)
  {
    format!("{}/{}", parts[0], parts[1])
  } else if parts.len() >= 3 {
    format!("{}-{}-{}", parts[0], parts[1], parts[2])
  } else {
    parts.join("-")
  };
  if has_circa {
    date.push('~');
  }
  *value = Value::Array(vec![Value::String(
    date
  )]);
  map.remove("date-circa");
}

fn bibtex_key_for(
  map: &Map<String, Value>,
  idx: usize,
  counts: &mut std::collections::HashMap<
    String,
    usize
  >
) -> String {
  let author = map
    .get("author")
    .and_then(Value::as_array)
    .and_then(|items| items.first())
    .and_then(Value::as_str)
    .unwrap_or("");
  let family = author
    .split(',')
    .next()
    .unwrap_or("")
    .split_whitespace()
    .next()
    .unwrap_or("");
  let family = family
    .chars()
    .filter(|c| c.is_ascii_alphanumeric())
    .collect::<String>()
    .to_lowercase();
  let date = map
    .get("date")
    .and_then(Value::as_array)
    .and_then(|items| items.first())
    .and_then(Value::as_str)
    .unwrap_or("");
  let year = date
    .chars()
    .filter(|c| c.is_ascii_digit())
    .collect::<String>();
  let year = if year.len() >= 4 {
    year[..4].to_string()
  } else {
    String::new()
  };
  if family.is_empty() || year.is_empty() {
    return format!("citeotter{idx}");
  }
  let base = format!("{family}{year}");
  let count = counts.entry(base.clone()).or_insert(0);
  let suffix = (*count as u8 + b'a') as char;
  *count += 1;
  format!("{base}{suffix}")
}
