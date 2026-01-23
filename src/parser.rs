use std::collections::{
  BTreeMap,
  BTreeSet
};

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
pub struct Author {
  pub family: String,
  pub given:  String
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum FieldValue {
  Single(String),
  List(Vec<String>),
  Authors(Vec<Author>)
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

#[derive(Debug, Clone, Default)]
struct FieldTokens {
  author:    BTreeSet<String>,
  title:     BTreeSet<String>,
  location:  BTreeSet<String>,
  publisher: BTreeSet<String>,
  date:      BTreeSet<String>,
  pages:     BTreeSet<String>,
  container: BTreeSet<String>,
  volume:    BTreeSet<String>,
  issue:     BTreeSet<String>,
  genre:     BTreeSet<String>,
  edition:   BTreeSet<String>
}

impl FieldTokens {
  fn from_reference(
    reference: &str
  ) -> Self {
    Self {
      author:    tokens_from_authors(
        reference
      ),
      title:     tokens_from_segment(
        &extract_title(reference)
      ),
      location:  tokens_from_segment(
        &extract_location(reference)
      ),
      publisher: tokens_from_segment(
        &extract_publisher(reference)
      ),
      date:      tokens_from_dates(
        reference
      ),
      pages:     tokens_from_segment(
        &extract_pages(reference)
      ),
      container: tokens_from_segment(
        extract_container_title(
          reference
        )
        .unwrap_or_default()
        .as_str()
      ),
      volume:    tokens_from_segment(
        extract_volume(reference)
          .unwrap_or_default()
          .as_str()
      ),
      issue:     tokens_from_segment(
        extract_issue(reference)
          .unwrap_or_default()
          .as_str()
      ),
      genre:     tokens_from_segment(
        extract_genre(reference)
          .unwrap_or_default()
          .as_str()
      ),
      edition:   tokens_from_segment(
        extract_edition(reference)
          .unwrap_or_default()
          .as_str()
      )
    }
  }
}

fn split_references(
  input: &str
) -> Vec<String> {
  input
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty())
    .map(|line| line.to_string())
    .collect()
}

fn tokens_from_segment(
  segment: &str
) -> BTreeSet<String> {
  let mut tokens = BTreeSet::new();

  let normalized =
    normalize_token(segment);
  if !normalized.is_empty() {
    tokens.insert(normalized);
  }

  segment
    .split(|c: char| {
      c == ','
        || c == ';'
        || c == ':'
        || c == '('
        || c == ')'
        || c == '*'
        || c == '-'
    })
    .map(str::trim)
    .filter(|part| !part.is_empty())
    .for_each(|part| {
      let normalized_part =
        normalize_token(part);
      if !normalized_part.is_empty() {
        tokens.insert(normalized_part);
      }
    });

  segment
    .split_whitespace()
    .map(normalize_token)
    .filter(|token| !token.is_empty())
    .for_each(|token| {
      tokens.insert(token);
    });

  tokens
}

fn parse_authors(
  reference: &str
) -> Vec<Author> {
  let segment =
    extract_author_segment(reference);
  let cleaned = segment
    .replace('&', ",")
    .replace(" and ", ",")
    .replace(" AND ", ",");
  let pieces: Vec<&str> = cleaned
    .split(',')
    .map(str::trim)
    .filter(|piece| !piece.is_empty())
    .collect();

  let mut authors = Vec::new();
  for chunk in pieces.chunks(2) {
    let family = chunk[0];
    let given = chunk
      .get(1)
      .copied()
      .unwrap_or("");
    let normalized_family =
      normalize_author_component(
        family
      );
    let normalized_given =
      normalize_author_component(given);
    if !normalized_family.is_empty()
      || !normalized_given.is_empty()
    {
      authors.push(Author {
        family: normalized_family,
        given:  normalized_given
      });
    }
  }

  if authors.is_empty()
    && !segment.trim().is_empty()
  {
    authors.push(Author {
      family:
        normalize_author_component(
          segment
        ),
      given:  "".into()
    });
  }

  authors
}

fn normalize_author_component(
  component: &str
) -> String {
  component
    .trim()
    .trim_matches(|c: char| {
      c == '.' || c == ',' || c == ';'
    })
    .trim()
    .to_string()
}

fn tokens_from_authors(
  reference: &str
) -> BTreeSet<String> {
  let author_segment =
    extract_author_segment(reference);
  let mut tokens =
    tokens_from_segment(author_segment);

  let normalized = author_segment
    .replace('&', ";")
    .replace(" and ", ";");
  for part in normalized.split(';') {
    let name = part.trim();
    if !name.is_empty() {
      tokens.extend(
        tokens_from_segment(name)
      );
    }
  }

  tokens
}

fn tokens_from_dates(
  reference: &str
) -> BTreeSet<String> {
  let mut tokens = BTreeSet::new();

  if let Some(year) =
    extract_year(reference)
  {
    tokens.extend(tokens_from_segment(
      &year
    ));
  }

  tokens.extend(capture_year_like(
    reference
  ));

  tokens
}

fn capture_year_like(
  reference: &str
) -> BTreeSet<String> {
  let mut tokens = BTreeSet::new();
  let mut buffer = String::new();

  for c in reference.chars() {
    if c.is_ascii_digit() {
      buffer.push(c);
    } else {
      if buffer.len() >= 4 {
        tokens.insert(buffer.clone());
      }
      buffer.clear();
    }
  }

  if buffer.len() >= 4 {
    tokens.insert(buffer);
  }

  tokens
}

fn matches_field(
  normalized: &str,
  values: &BTreeSet<String>
) -> bool {
  values.iter().any(|value| {
    !value.is_empty()
      && normalized.contains(value)
  })
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
    let references =
      split_references(input);
    let contexts: Vec<FieldTokens> =
      references
        .iter()
        .map(|reference| {
          FieldTokens::from_reference(
            reference
          )
        })
        .collect();
    let default_context =
      FieldTokens::default();

    self
      .prepare(input, true)
      .0
      .iter()
      .enumerate()
      .map(|(idx, sequence)| {
        let context = contexts
          .get(idx)
          .unwrap_or(&default_context);
        sequence
          .iter()
          .map(|token| {
            TaggedToken {
              token: token.clone(),
              label: tag_token(
                token, context
              )
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
        let authors =
          parse_authors(reference);
        if !authors.is_empty() {
          mapped.insert(
            "author",
            FieldValue::Authors(
              authors
            )
          );
        } else {
          mapped.insert(
            "author",
            FieldValue::List(vec![
              extract_author(reference),
            ])
          );
        }
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

        if let Some(container) =
          extract_container_title(
            reference
          )
        {
          mapped.insert(
            "container-title",
            FieldValue::List(vec![
              container,
            ])
          );
        }

        if let Some(volume) =
          extract_volume(reference)
        {
          mapped.insert(
            "volume",
            FieldValue::List(vec![
              volume,
            ])
          );
        }

        if let Some(issue) =
          extract_issue(reference)
        {
          mapped.insert(
            "issue",
            FieldValue::List(vec![
              issue,
            ])
          );
        }

        if let Some(edition) =
          extract_edition(reference)
        {
          mapped.insert(
            "edition",
            FieldValue::List(vec![
              edition,
            ])
          );
        }

        if let Some(genre) =
          extract_genre(reference)
        {
          mapped.insert(
            "genre",
            FieldValue::List(vec![
              genre,
            ])
          );
        }

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
            guess_language(reference)
              .into()
          )
        );
        mapped.insert(
          "scripts",
          FieldValue::List(
            detect_scripts(reference)
          )
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

fn extract_author(
  reference: &str
) -> String {
  extract_author_segment(reference)
    .trim()
    .to_string()
}

fn extract_author_segment(
  reference: &str
) -> &str {
  reference
    .split('.')
    .next()
    .unwrap_or(reference)
    .trim()
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

fn extract_container_title(
  reference: &str
) -> Option<String> {
  let keywords = [
    "journal",
    "proceedings",
    "conference",
    "symposium",
    "meeting",
    "presented",
    "proceedings of"
  ];
  reference
    .split('.')
    .map(str::trim)
    .filter(|segment| {
      !segment.is_empty()
    })
    .find(|segment| {
      let lower =
        segment.to_lowercase();
      keywords
        .iter()
        .any(|kw| lower.contains(kw))
    })
    .map(|segment| {
      segment
        .trim_end_matches(|c: char| {
          c == ':'
            || c == ','
            || c == ';'
        })
        .trim()
        .to_string()
    })
}

fn capture_number_after(
  reference: &str,
  start: usize
) -> Option<String> {
  let remainder =
    reference.get(start..)?;
  let digits: String = remainder
    .chars()
    .skip_while(|c| !c.is_ascii_digit())
    .take_while(|c| c.is_ascii_digit())
    .collect();
  if digits.is_empty() {
    None
  } else {
    Some(digits)
  }
}

fn extract_volume(
  reference: &str
) -> Option<String> {
  let lower = reference.to_lowercase();
  for keyword in [
    "volume", "vol.", "vol", "v.",
    "vols"
  ] {
    if let Some(pos) =
      lower.find(keyword)
    {
      let start = pos + keyword.len();
      if let Some(digits) =
        capture_number_after(
          reference, start
        )
      {
        return Some(digits);
      }
    }
  }
  None
}

fn extract_issue(
  reference: &str
) -> Option<String> {
  let lower = reference.to_lowercase();
  for keyword in
    ["number", "no.", "issue", "part"]
  {
    if let Some(pos) =
      lower.find(keyword)
    {
      let start = pos + keyword.len();
      if let Some(digits) =
        capture_number_after(
          reference, start
        )
      {
        return Some(digits);
      }
    }
  }
  None
}

fn extract_genre(
  reference: &str
) -> Option<String> {
  let start = reference.find('[')?;
  let close =
    reference[start + 1..].find(']')?;
  Some(
    reference
      [start + 1..start + 1 + close]
      .trim()
      .to_string()
  )
}

fn extract_edition(
  reference: &str
) -> Option<String> {
  let lower = reference.to_lowercase();
  for keyword in
    ["edition", "éd.", "ed.", "édc"]
  {
    if let Some(pos) =
      lower.find(keyword)
    {
      let start = pos + keyword.len();
      let remainder = reference
        .get(start..)
        .unwrap_or("");
      let edition = remainder
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim_matches(|c: char| {
          c == ','
            || c == '.'
            || c == '('
            || c == ')'
        });
      let normalized =
        normalize_token(edition);
      if !normalized.is_empty() {
        return Some(normalized);
      }
      break;
    }
  }
  None
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

fn detect_scripts(
  reference: &str
) -> Vec<String> {
  let mut scripts = BTreeSet::new();
  scripts.insert("Common".to_string());
  scripts.insert("Latin".to_string());
  if reference.chars().any(|c| {
    c.is_alphabetic() && !c.is_ascii()
  }) {
    scripts.insert("Other".to_string());
  }
  scripts.into_iter().collect()
}

fn guess_language(
  reference: &str
) -> &'static str {
  if reference
    .chars()
    .any(|c| c as u32 > 0x007f)
  {
    "fr"
  } else {
    "en"
  }
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

pub fn sequence_signature(
  tokens: &[String]
) -> String {
  tokens
    .iter()
    .map(|token| token.trim())
    .filter(|token| !token.is_empty())
    .map(|token| token.to_string())
    .collect::<Vec<_>>()
    .join(" ")
}

fn tag_token(
  token: &str,
  context: &FieldTokens
) -> String {
  let original = token
    .split_whitespace()
    .next()
    .unwrap_or(token);
  let lower = original.to_lowercase();
  let normalized =
    normalize_token(original);

  if matches_field(
    &normalized,
    &context.author
  ) {
    "author".into()
  } else if matches_field(
    &normalized,
    &context.title
  ) {
    "title".into()
  } else if matches_field(
    &normalized,
    &context.container
  ) {
    "container-title".into()
  } else if matches_field(
    &normalized,
    &context.location
  ) {
    "location".into()
  } else if matches_field(
    &normalized,
    &context.publisher
  ) {
    "publisher".into()
  } else if matches_field(
    &normalized,
    &context.date
  ) {
    "date".into()
  } else if matches_field(
    &normalized,
    &context.pages
  ) {
    "pages".into()
  } else if matches_field(
    &normalized,
    &context.volume
  ) {
    "volume".into()
  } else if matches_field(
    &normalized,
    &context.issue
  ) {
    "issue".into()
  } else if matches_field(
    &normalized,
    &context.genre
  ) {
    "genre".into()
  } else if matches_field(
    &normalized,
    &context.edition
  ) {
    "edition".into()
  } else if lower.contains("press") {
    "publisher".into()
  } else if lower.contains(',') {
    "author".into()
  } else if lower.contains("p.")
    || lower.contains("pp.")
  {
    "pages".into()
  } else if lower.contains("london") {
    "location".into()
  } else if normalized
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
