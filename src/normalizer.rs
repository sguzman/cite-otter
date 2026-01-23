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
          append_field(
            map,
            "container-title",
            journal
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

fn is_repeater(value: &str) -> bool {
  let allowed = ['-', '.', ',', ' '];
  !value.is_empty()
    && value.chars().all(|c| {
      c.is_whitespace()
        || allowed.contains(&c)
    })
}
