pub mod names {
  #[derive(Debug, Clone)]
  pub struct Normalizer;

  impl Default for Normalizer {
    fn default() -> Self {
      Self::new()
    }
  }

  impl Normalizer {
    pub fn new() -> Self {
      Self
    }

    pub fn normalize(
      &self,
      input: &str,
      prev: Option<&[&str]>
    ) -> Vec<String> {
      let trimmed = input.trim();
      let cleaned = trimmed
        .trim_end_matches(',')
        .to_string();

      if let Some(first) = prev
        .and_then(|previous| {
          previous.first()
        })
        .filter(|_| {
          super::is_repeater(trimmed)
        })
      {
        return vec![first.to_string()];
      }

      vec![cleaned]
    }
  }
}

pub mod abbreviations {
  use std::collections::HashMap;
  use std::fs;
  use std::path::Path;

  #[derive(Debug, Clone, Default)]
  pub struct AbbreviationMap {
    entries: HashMap<String, String>
  }

  impl AbbreviationMap {
    pub fn new() -> Self {
      Self {
        entries: HashMap::new()
      }
    }

    pub fn load_from_str(
      text: &str
    ) -> Self {
      let mut map = Self::new();
      for line in text.lines() {
        if let Some((key, value)) =
          parse_line(line)
        {
          map
            .entries
            .insert(key, value);
        }
      }
      map
    }

    pub fn load_from_file(
      path: &Path
    ) -> std::io::Result<Self> {
      let content =
        fs::read_to_string(path)?;
      Ok(Self::load_from_str(&content))
    }

    pub fn insert(
      &mut self,
      key: impl Into<String>,
      value: impl Into<String>
    ) {
      let key =
        normalize_key(&key.into());
      self
        .entries
        .insert(key, value.into());
    }

    pub fn len(&self) -> usize {
      self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
      self.entries.is_empty()
    }

    pub fn expand(
      &self,
      value: &str
    ) -> String {
      let key = normalize_key(value);
      self
        .entries
        .get(&key)
        .cloned()
        .unwrap_or_else(|| {
          value.trim().to_string()
        })
    }
  }

  fn parse_line(
    line: &str
  ) -> Option<(String, String)> {
    let trimmed = line.trim();
    if trimmed.is_empty()
      || trimmed.starts_with('#')
    {
      return None;
    }
    let mut parts =
      trimmed.splitn(2, |c| {
        c == '\t'
          || c == ','
          || c == '='
      });
    let key = parts.next()?.trim();
    let value = parts.next()?.trim();
    if key.is_empty()
      || value.is_empty()
    {
      return None;
    }
    Some((
      normalize_key(key),
      value.to_string()
    ))
  }

  fn normalize_key(
    value: &str
  ) -> String {
    value
      .trim()
      .trim_end_matches('.')
      .split_whitespace()
      .collect::<Vec<_>>()
      .join(" ")
      .to_lowercase()
  }
}

pub mod location {
  #[derive(Debug, Clone)]
  pub struct Normalizer;

  impl Default for Normalizer {
    fn default() -> Self {
      Self::new()
    }
  }

  impl Normalizer {
    pub fn new() -> Self {
      Self
    }

    pub fn normalize(
      &self,
      input: &str
    ) -> (String, Option<String>) {
      let trimmed = input.trim();
      if let Some((before, after)) =
        trimmed.split_once(':')
      {
        (
          before.trim().to_string(),
          Some(
            after.trim().to_string()
          )
        )
      } else {
        (trimmed.to_string(), None)
      }
    }
  }
}

pub mod container {
  #[derive(Debug, Clone)]
  pub struct Normalizer;

  impl Default for Normalizer {
    fn default() -> Self {
      Self::new()
    }
  }

  impl Normalizer {
    pub fn new() -> Self {
      Self
    }

    pub fn normalize(
      &self,
      input: &str
    ) -> String {
      let mut value = input.trim();
      for prefix in &[
        "in ",
        "In ",
        "of ",
        "Presented at ",
        "presented at "
      ] {
        if value.starts_with(prefix) {
          value = value[prefix.len()..]
            .trim();
        }
      }
      value
        .trim_end_matches(|c: char| {
          c == ',' || c == '.'
        })
        .trim()
        .to_string()
    }
  }
}

pub mod journal {
  use serde_json::map::Entry;
  use serde_json::{
    Map,
    Value
  };

  use super::abbreviations::AbbreviationMap;

  #[derive(Debug, Clone)]
  pub struct Normalizer;

  impl Default for Normalizer {
    fn default() -> Self {
      Self::new()
    }
  }

  impl Normalizer {
    pub fn new() -> Self {
      Self
    }

    pub fn normalize(
      &self,
      map: &mut Map<String, Value>
    ) {
      self.normalize_with_abbrev(
        map,
        &AbbreviationMap::default()
      );
    }

    pub fn normalize_with_abbrev(
      &self,
      map: &mut Map<String, Value>,
      abbreviations: &AbbreviationMap
    ) {
      if let Some(value) =
        map.remove("journal")
      {
        map.insert(
          "type".into(),
          Value::String(
            "article-journal".into()
          )
        );
        let journals =
          extract_strings(value);
        for journal in journals {
          let expanded = abbreviations
            .expand(&journal);
          append_field(
            map,
            "container-title",
            expanded
          );
        }
      }
    }
  }

  fn extract_strings(
    value: Value
  ) -> Vec<String> {
    match value {
      | Value::Array(items) => {
        items
          .into_iter()
          .filter_map(|item| {
            item
              .as_str()
              .map(|s| s.to_string())
          })
          .collect()
      }
      | Value::String(text) => {
        vec![text]
      }
      | _ => Vec::new()
    }
  }

  fn append_field(
    map: &mut Map<String, Value>,
    key: &str,
    text: String
  ) {
    match map.entry(key.to_string()) {
      | Entry::Vacant(entry) => {
        entry.insert(Value::Array(
          vec![Value::String(text)]
        ));
      }
      | Entry::Occupied(mut entry) => {
        if let Value::Array(array) =
          entry.get_mut()
        {
          array
            .push(Value::String(text));
        }
      }
    }
  }
}

use std::path::Path;

use abbreviations::AbbreviationMap;
use journal::Normalizer as JournalNormalizer;
use serde_json::{
  Map,
  Value
};

use crate::parser::FieldValue;

#[derive(Debug, Clone)]
pub struct NormalizationConfig {
  journal:   AbbreviationMap,
  publisher: AbbreviationMap,
  container: AbbreviationMap,
  language:  AbbreviationMap,
  scripts:   AbbreviationMap
}

impl Default for NormalizationConfig {
  fn default() -> Self {
    Self {
      journal:
        AbbreviationMap::default(),
      publisher:
        AbbreviationMap::default(),
      container:
        AbbreviationMap::default(),
      language:
        AbbreviationMap::default(),
      scripts:
        AbbreviationMap::default()
    }
  }
}

impl NormalizationConfig {
  pub fn load_from_dir(
    dir: &Path
  ) -> std::io::Result<Self> {
    Ok(Self {
      journal:   load_abbrev(
        dir,
        "journal-abbrev.txt"
      )?,
      publisher: load_abbrev(
        dir,
        "publisher-abbrev.txt"
      )?,
      container: load_abbrev(
        dir,
        "container-abbrev.txt"
      )?,
      language:  load_abbrev(
        dir,
        "language-locale.txt"
      )?,
      scripts:   load_abbrev(
        dir,
        "script-locale.txt"
      )?
    })
  }

  pub fn with_journal_abbrev(
    mut self,
    abbreviations: AbbreviationMap
  ) -> Self {
    self.journal = abbreviations;
    self
  }

  pub fn with_publisher_abbrev(
    mut self,
    abbreviations: AbbreviationMap
  ) -> Self {
    self.publisher = abbreviations;
    self
  }

  pub fn with_container_abbrev(
    mut self,
    abbreviations: AbbreviationMap
  ) -> Self {
    self.container = abbreviations;
    self
  }

  pub fn with_language_locale(
    mut self,
    abbreviations: AbbreviationMap
  ) -> Self {
    self.language = abbreviations;
    self
  }

  pub fn with_script_locale(
    mut self,
    abbreviations: AbbreviationMap
  ) -> Self {
    self.scripts = abbreviations;
    self
  }

  pub fn apply_to_map(
    &self,
    map: &mut Map<String, Value>
  ) {
    JournalNormalizer::new()
      .normalize_with_abbrev(
        map,
        &self.journal
      );
    expand_field(
      map,
      "publisher",
      &self.publisher
    );
    expand_field(
      map,
      "container-title",
      &self.container
    );
    expand_field(
      map,
      "language",
      &self.language
    );
    expand_field(
      map,
      "scripts",
      &self.scripts
    );
  }

  pub fn apply_to_fields(
    &self,
    map: &mut std::collections::BTreeMap<
      String,
      FieldValue
    >
  ) {
    expand_field_value(
      map,
      "journal",
      &self.journal
    );
    expand_field_value(
      map,
      "publisher",
      &self.publisher
    );
    expand_field_value(
      map,
      "container-title",
      &self.container
    );
    expand_field_value(
      map,
      "language",
      &self.language
    );
    expand_field_value(
      map,
      "scripts",
      &self.scripts
    );
  }
}

fn load_abbrev(
  dir: &Path,
  filename: &str
) -> std::io::Result<AbbreviationMap> {
  let path = dir.join(filename);
  if path.exists() {
    AbbreviationMap::load_from_file(
      &path
    )
  } else {
    Ok(AbbreviationMap::default())
  }
}

fn expand_field(
  map: &mut Map<String, Value>,
  key: &str,
  abbreviations: &AbbreviationMap
) {
  let Some(value) = map.get_mut(key)
  else {
    return;
  };
  match value {
    | Value::String(text) => {
      let expanded =
        abbreviations.expand(text);
      *text = expanded;
    }
    | Value::Array(items) => {
      for item in items {
        if let Value::String(text) =
          item
        {
          let expanded =
            abbreviations.expand(text);
          *text = expanded;
        }
      }
    }
    | _ => {}
  }
}

fn expand_field_value(
  map: &mut std::collections::BTreeMap<
    String,
    FieldValue
  >,
  key: &str,
  abbreviations: &AbbreviationMap
) {
  let Some(value) = map.get_mut(key)
  else {
    return;
  };
  match value {
    | FieldValue::Single(text) => {
      let expanded =
        abbreviations.expand(text);
      *text = expanded;
    }
    | FieldValue::List(items) => {
      for item in items {
        let expanded =
          abbreviations.expand(item);
        *item = expanded;
      }
    }
    | FieldValue::Authors(_) => {}
  }
}

fn is_repeater(value: &str) -> bool {
  let allowed = ['-', '.', ',', ' '];
  !value.is_empty()
    && value.chars().all(|c| {
      c.is_whitespace()
        || allowed.contains(&c)
    })
}
