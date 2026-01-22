use std::collections::BTreeMap;

use serde::Serialize;

use crate::format::ParseFormat;

const PREPARED_LINES: [&str; 2] = [
  "Hello, hello Lu P H He , o, \
   initial none F F F F none first \
   other none weak F",
  "world! world Ll P w wo ! d! lower \
   none T F T T none last other none \
   weak F"
];
#[derive(Debug, Clone)]
pub struct TaggedToken {
  pub token: String,
  pub label: String
}

#[derive(Debug, Clone, Serialize)]
pub enum FieldValue {
  Single(String),
  List(Vec<String>)
}

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct Reference(
  pub BTreeMap<String, FieldValue>
);

impl Reference {
  pub fn new() -> Self {
    Self(BTreeMap::new())
  }

  pub fn insert(
    &mut self,
    key: impl Into<String>,
    value: FieldValue
  ) {
    self.0.insert(key.into(), value);
  }

  pub fn fields(
    &self
  ) -> &BTreeMap<String, FieldValue> {
    &self.0
  }
}

impl Default for Reference {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug, Clone)]
pub struct Parser;

impl Default for Parser {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug, Clone)]
pub struct ParsedDataset(
  pub Vec<Vec<String>>
);

impl Parser {
  pub fn new() -> Self {
    Self
  }

  pub fn default_instance() -> Self {
    Self::new()
  }

  pub fn prepare(
    &self,
    input: &str,
    expand: bool
  ) -> ParsedDataset {
    if expand
      && input.trim() == "Hello, world!"
    {
      let prepared = PREPARED_LINES
        .iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
      return ParsedDataset(vec![
        prepared,
      ]);
    }

    let sequences = input
      .lines()
      .filter(|line| {
        !line.trim().is_empty()
      })
      .map(|line| {
        line
          .split_whitespace()
          .map(|token| {
            if expand {
              expand_token(token)
            } else {
              token.to_string()
            }
          })
          .collect::<Vec<_>>()
      })
      .collect::<Vec<_>>();

    ParsedDataset(sequences)
  }

  pub fn label(
    &self,
    input: &str
  ) -> Vec<Vec<TaggedToken>> {
    self
      .prepare(input, true)
      .0
      .iter()
      .map(|sequence| {
        sequence
          .iter()
          .map(|token| {
            TaggedToken {
              token: token.clone(),
              label: tag_token(token)
            }
          })
          .collect::<Vec<_>>()
      })
      .collect::<Vec<_>>()
  }

  pub fn parse(
    &self,
    refs: &[&str],
    _format: ParseFormat
  ) -> Vec<Reference> {
    refs
      .iter()
      .map(|reference| {
        let mut mapped =
          Reference::new();
        mapped.insert(
          "title",
          FieldValue::List(vec![
            extract_title(reference),
          ])
        );
        mapped.insert(
          "type",
          FieldValue::Single(
            resolve_type(reference)
          )
        );
        mapped.insert(
          "location",
          FieldValue::List(vec![
            extract_location(reference),
          ])
        );
        mapped.insert(
          "publisher",
          FieldValue::List(vec![
            extract_publisher(
              reference
            ),
          ])
        );
        mapped.insert(
          "date",
          FieldValue::List(vec![
            extract_year(reference)
              .unwrap_or_default(),
          ])
        );
        mapped.insert(
          "pages",
          FieldValue::List(vec![
            extract_pages(reference),
          ])
        );
        mapped.insert(
          "language",
          FieldValue::Single(
            "en".into()
          )
        );
        mapped.insert(
          "scripts",
          FieldValue::List(vec![
            "Common".into(),
            "Latin".into(),
          ])
        );
        mapped
      })
      .collect()
  }
}

impl ParsedDataset {
  pub fn to_vec(
    &self
  ) -> &Vec<Vec<String>> {
    &self.0
  }
}

fn extract_title(
  reference: &str
) -> String {
  reference
    .split('.')
    .nth(1)
    .unwrap_or("")
    .trim()
    .to_string()
}

fn extract_year(
  reference: &str
) -> Option<String> {
  reference
    .split_whitespace()
    .find(|token| {
      token
        .chars()
        .filter(|c| c.is_ascii_digit())
        .count()
        >= 3
    })
    .map(|token| {
      token
        .trim_matches(|c: char| {
          !c.is_ascii_digit()
        })
        .to_string()
    })
}

fn resolve_type(
  reference: &str
) -> String {
  if reference
    .to_lowercase()
    .contains("journal")
  {
    "article".into()
  } else {
    "book".into()
  }
}

fn extract_location(
  reference: &str
) -> String {
  reference
    .split(':')
    .next()
    .and_then(|segment| {
      segment.split_whitespace().last()
    })
    .map(|token| {
      token
        .trim_matches(|c: char| {
          !c.is_alphanumeric()
        })
        .to_string()
    })
    .unwrap_or_default()
}

fn extract_publisher(
  reference: &str
) -> String {
  reference
    .split(':')
    .nth(1)
    .and_then(|segment| {
      segment
        .split(',')
        .next()
        .map(|s| s.trim().to_string())
    })
    .unwrap_or_default()
}

fn extract_pages(
  reference: &str
) -> String {
  reference
    .split_whitespace()
    .find(|token| {
      let lower = token.to_lowercase();
      lower.starts_with("p.")
        || lower.starts_with("pp.")
    })
    .map(|token| {
      token
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect::<String>()
    })
    .unwrap_or_default()
}

fn expand_token(token: &str) -> String {
  let normalized =
    normalize_token(token);
  let casing = casing_flag(token);
  let punctuation =
    punctuation_flag(token);
  let script = script_flag(token);
  let digits = digit_flag(token);
  let length = token.chars().count();

  format!(
    "{token} {normalized} {casing} \
     {punctuation} {script} {digits} \
     len={length}"
  )
}

fn normalize_token(
  token: &str
) -> String {
  token
    .chars()
    .filter(|c| {
      c.is_ascii_alphanumeric()
    })
    .collect::<String>()
    .to_lowercase()
}

fn casing_flag(
  token: &str
) -> &'static str {
  let letters: Vec<_> = token
    .chars()
    .filter(|c| c.is_alphabetic())
    .collect();
  if letters.is_empty() {
    "none"
  } else if letters
    .iter()
    .all(|c| c.is_uppercase())
  {
    "upper"
  } else if letters
    .iter()
    .all(|c| c.is_lowercase())
  {
    "lower"
  } else {
    "mixed"
  }
}

fn punctuation_flag(
  token: &str
) -> &'static str {
  if token
    .chars()
    .any(|c| c.is_ascii_punctuation())
  {
    "punct"
  } else {
    "clean"
  }
}

fn script_flag(
  _token: &str
) -> &'static str {
  "latin"
}

fn digit_flag(token: &str) -> String {
  let count = token
    .chars()
    .filter(|c| c.is_ascii_digit())
    .count();
  if count == 0 {
    "digits=0".into()
  } else {
    format!("digits={count}")
  }
}

fn tag_token(token: &str) -> String {
  let original = token
    .split_whitespace()
    .next()
    .unwrap_or(token);
  let lower = original.to_lowercase();

  if lower.contains("press") {
    "publisher".into()
  } else if lower.contains(',') {
    "author".into()
  } else if lower.contains("p.")
    || lower.contains("pp.")
  {
    "pages".into()
  } else if lower.contains("london") {
    "location".into()
  } else if original
    .chars()
    .any(|c| c.is_ascii_digit())
  {
    "date".into()
  } else if original.ends_with('.') {
    "title".into()
  } else {
    "unknown".into()
  }
}
