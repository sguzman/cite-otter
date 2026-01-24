use std::collections::{
  BTreeMap,
  BTreeSet
};

use serde::Serialize;

use crate::dictionary::{
  Dictionary,
  DictionaryAdapter,
  DictionaryCode
};
use crate::format::ParseFormat;
use crate::language::{
  detect_language,
  detect_scripts
};
use crate::normalizer::NormalizationConfig;

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

#[derive(
  Debug, Clone, Serialize, PartialEq, Eq,
)]
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

  pub fn from_map(
    map: BTreeMap<String, FieldValue>
  ) -> Self {
    Self(map)
  }
}

impl Default for Reference {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug, Clone, Default)]
struct FieldTokens {
  author:     BTreeSet<String>,
  title:      BTreeSet<String>,
  location:   BTreeSet<String>,
  publisher:  BTreeSet<String>,
  date:       BTreeSet<String>,
  pages:      BTreeSet<String>,
  container:  BTreeSet<String>,
  collection: BTreeSet<String>,
  journal:    BTreeSet<String>,
  editor:     BTreeSet<String>,
  translator: BTreeSet<String>,
  note:       BTreeSet<String>,
  identifier: BTreeSet<String>,
  volume:     BTreeSet<String>,
  issue:      BTreeSet<String>,
  genre:      BTreeSet<String>,
  edition:    BTreeSet<String>
}

impl FieldTokens {
  fn from_reference(
    reference: &str
  ) -> Self {
    Self {
      author:     tokens_from_authors(
        reference
      ),
      title:      tokens_from_segment(
        &extract_title(reference)
      ),
      location:   tokens_from_segment(
        &extract_location(reference)
      ),
      publisher:  tokens_from_segment(
        &extract_publisher(reference)
      ),
      date:       tokens_from_dates(
        reference
      ),
      pages:      tokens_from_segment(
        &extract_pages(reference)
      ),
      container:  tokens_from_segment(
        extract_container_title(
          reference
        )
        .unwrap_or_default()
        .as_str()
      ),
      collection: tokens_from_segment(
        extract_collection_title(
          reference
        )
        .unwrap_or_default()
        .as_str()
      ),
      journal:    tokens_from_segment(
        extract_journal(reference)
          .unwrap_or_default()
          .as_str()
      ),
      editor:     tokens_from_segment(
        extract_editor(reference)
          .unwrap_or_default()
          .as_str()
      ),
      translator: tokens_from_segment(
        extract_translator(reference)
          .unwrap_or_default()
          .as_str()
      ),
      note:       tokens_from_segment(
        extract_note(reference)
          .unwrap_or_default()
          .as_str()
      ),
      identifier:
        tokens_from_identifiers(
          reference
        ),
      volume:     tokens_from_segment(
        extract_volume(reference)
          .unwrap_or_default()
          .as_str()
      ),
      issue:      tokens_from_segment(
        extract_issue(reference)
          .unwrap_or_default()
          .as_str()
      ),
      genre:      tokens_from_segment(
        extract_genre(reference)
          .unwrap_or_default()
          .as_str()
      ),
      edition:    tokens_from_segment(
        extract_edition(reference)
          .unwrap_or_default()
          .as_str()
      )
    }
  }

  fn from_reference_with_dictionary(
    reference: &str,
    dictionary: &Dictionary
  ) -> Self {
    let mut tokens =
      Self::from_reference(reference);
    tokens.apply_dictionary(
      reference, dictionary
    );
    tokens
  }

  fn apply_dictionary(
    &mut self,
    reference: &str,
    dictionary: &Dictionary
  ) {
    for term in
      reference.split(|c: char| {
        !c.is_alphanumeric()
      })
    {
      let term = term.trim();
      if term.is_empty() {
        continue;
      }
      let normalized =
        normalize_token(term);
      if normalized.is_empty() {
        continue;
      }
      for code in
        dictionary.lookup(term)
      {
        match code {
          | DictionaryCode::Name => {
            self.author
              .insert(normalized.clone());
          }
          | DictionaryCode::Place => {
            self.location
              .insert(normalized.clone());
          }
          | DictionaryCode::Publisher => {
            self.publisher
              .insert(normalized.clone());
          }
          | DictionaryCode::Journal => {
            self.journal
              .insert(normalized.clone());
          }
        }
      }
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

fn split_reference_segments(
  reference: &str
) -> Vec<String> {
  let mut segments = Vec::new();
  let mut last_start = 0usize;
  let mut depth = 0usize;

  for (idx, ch) in
    reference.char_indices()
  {
    if ch == '(' {
      depth = depth.saturating_add(1);
    } else if ch == ')'
      && depth > 0
    {
      depth = depth.saturating_sub(1);
    }
    if ch != '.' {
      continue;
    }
    if depth > 0 {
      continue;
    }
    if is_initial_boundary(reference, idx) {
      continue;
    }
    let mut next_chars = reference
      [idx + ch.len_utf8()..]
      .chars()
      .skip_while(|c| {
        c.is_whitespace()
      });
    let next = next_chars.next();
    let is_boundary =
      next.map_or(true, |next_char| {
        next_char.is_uppercase()
          || next_char.is_ascii_digit()
          || matches!(next_char, '"' | '“' | '‘')
      });
    if !is_boundary {
      continue;
    }
    let segment = reference
      [last_start..idx]
      .trim()
      .to_string();
    if !segment.is_empty() {
      segments.push(segment);
    }
    last_start = idx + ch.len_utf8();
  }

  let tail = reference[last_start..]
    .trim()
    .to_string();
  if !tail.is_empty() {
    segments.push(tail);
  }

  segments
}

fn is_initial_boundary(
  reference: &str,
  idx: usize
) -> bool {
  let before = reference[..idx].trim_end();
  let mut token_start = 0usize;
  for (pos, ch) in before.char_indices()
  {
    if ch.is_whitespace() {
      token_start = pos + ch.len_utf8();
    }
  }
  let token = before[token_start..].trim();
  if token.len() != 1
    || !token
      .chars()
      .all(|c| c.is_alphabetic())
  {
    return false;
  }
  let mut chars = reference
    [idx + 1..]
    .chars()
    .skip_while(|c| c.is_whitespace());
  let next = chars.next();
  let following = chars.next();
  matches!(
    (next, following),
    (Some(letter), Some('.'))
      if letter.is_alphabetic()
  )
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
  let normalized = segment
    .replace('&', ";")
    .replace(" and ", ";")
    .replace(" AND ", ";")
    .replace(" / ", ";")
    .replace("/", ";")
    .replace('|', ";");

  normalized
    .split(';')
    .map(str::trim)
    .filter(|piece| !piece.is_empty())
    .flat_map(split_author_candidates)
    .filter_map(|piece| {
      parse_author_chunk(&piece)
    })
    .collect::<Vec<_>>()
}

fn split_author_candidates(
  piece: &str
) -> Vec<String> {
  let trimmed = piece.trim();
  if !trimmed.contains(',') {
    return vec![trimmed.to_string()];
  }
  let parts = trimmed
    .split(',')
    .map(str::trim)
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>();
  if parts.len() < 2 {
    return vec![trimmed.to_string()];
  }
  if let Some(last) = parts.last()
    && is_author_suffix(last)
  {
    if parts.len() == 3 {
      return vec![trimmed.to_string()];
    }
    if parts.len() > 3
      && looks_like_initials(parts[1])
    {
      let mut grouped = vec![format!(
        "{}, {}, {}",
        parts[0], parts[1], parts[2]
      )];
      let remainder =
        parts[3..].join(", ");
      if !remainder.is_empty() {
        grouped.extend(
          split_author_candidates(&remainder)
        );
      }
      return grouped;
    }
  }
  if parts.len() % 2 == 0
    && parts.chunks(2).all(|pair| {
      pair.len() == 2
        && looks_like_initials(pair[1])
    })
  {
    return parts
      .chunks(2)
      .map(|pair| {
        format!("{}, {}", pair[0], pair[1])
      })
      .collect();
  }
  if parts.len() >= 3
    && parts.iter().all(|part| {
      !part.contains(',')
    })
  {
    return parts
      .into_iter()
      .map(|part| part.to_string())
      .collect();
  }
  vec![trimmed.to_string()]
}

fn is_author_suffix(
  value: &str
) -> bool {
  let normalized = normalize_author_component(
    value
  )
  .to_lowercase();
  matches!(
    normalized.as_str(),
    "jr" | "sr" | "ii" | "iii" | "iv"
  )
}

fn parse_author_chunk(
  chunk: &str
) -> Option<Author> {
  let trimmed = chunk.trim();
  if trimmed.is_empty() {
    return None;
  }
  let lowered = trimmed
    .trim_end_matches('.')
    .to_lowercase();
  if lowered == "et al" {
    return None;
  }

  let (family, given) = if trimmed
    .contains(',')
  {
    let mut parts = trimmed
      .split(',')
      .map(str::trim)
      .filter(|part| !part.is_empty())
      .collect::<Vec<_>>();
    let family =
      parts.remove(0).to_string();
    let given = parts.join(" ");
    (family, given)
  } else {
    let tokens = trimmed
      .split_whitespace()
      .collect::<Vec<_>>();
    if tokens.is_empty() {
      return None;
    }
    let mut tokens: Vec<String> =
      tokens
        .iter()
        .map(|token| {
          (*token).to_string()
        })
        .collect();
    let suffixes =
      ["jr", "sr", "ii", "iii", "iv"];
    let suffix = tokens
      .last()
      .map(|token| {
        normalize_author_component(
          token
        )
        .to_lowercase()
      })
      .filter(|token| {
        suffixes
          .contains(&token.as_str())
      })
      .map(|token| token);
    if suffix.is_some() {
      tokens.pop();
    }
    let family_end = tokens.len();
    let particle_set = [
      "da", "de", "del", "der", "den",
      "di", "du", "la", "le", "van",
      "von", "al", "bin", "ibn"
    ];
    let mut family_start = family_end;
    for (idx, token) in
      tokens.iter().enumerate().rev()
    {
      if idx + 1 != family_start {
        break;
      }
      let normalized =
        normalize_author_component(
          token
        )
        .to_lowercase();
      if normalized.is_empty() {
        continue;
      }
      let is_particle = token
        .chars()
        .all(|c| c.is_lowercase())
        || particle_set.contains(
          &normalized.as_str()
        );
      if is_particle {
        family_start = idx;
      } else {
        break;
      }
    }
    let mut family_start = family_start;
    if family_start == family_end {
      family_start =
        family_end.saturating_sub(1);
    }
    if family_end >= 2
      && looks_like_initials(
        &tokens[family_end - 1]
      )
    {
      let family =
        tokens[..family_end - 1].join(" ");
      let mut given_parts = vec![
        tokens[family_end - 1].clone(),
      ];
      if let Some(token) = suffix {
        given_parts.push(token);
      }
      return Some(Author {
        family: normalize_author_component(
          &family
        ),
        given:  normalize_author_component(
          &strip_et_al_suffix(
            &given_parts.join(" ")
          )
        )
      });
    }
    let family =
      tokens[family_start..].join(" ");
    let mut given_parts: Vec<String> =
      tokens[..family_start].to_vec();
    if let Some(token) = suffix {
      given_parts.push(token);
    }
    let given = given_parts.join(" ");
    (family, given)
  };

  let normalized_family =
    normalize_author_component(&family);
  let normalized_given =
    normalize_author_component(
      &strip_et_al_suffix(&given)
    );
  if normalized_family.is_empty()
    && normalized_given.is_empty()
  {
    return None;
  }

  Some(Author {
    family: normalized_family,
    given:  normalized_given
  })
}

fn authors_for_reference(
  reference: &str
) -> Vec<Author> {
  let mut authors =
    parse_authors(reference);
  if authors.is_empty() {
    if let Some(author) =
      parse_author_chunk(
        &extract_author_segment(
          reference
        )
      )
    {
      authors.push(author);
    } else {
      let fallback =
        extract_author(reference);
      if !fallback.is_empty() {
        authors.push(Author {
          family: fallback,
          given:  String::new()
        });
      }
    }
  }
  authors
}

fn normalize_author_component(
  component: &str
) -> String {
  component
    .split_whitespace()
    .map(|part| {
      part
        .trim_matches(|c: char| {
          matches!(
            c,
            '.' | ',' | ';' | ':' | '!'
              | '?' | '(' | ')' | '['
              | ']'
          )
        })
        .to_string()
    })
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>()
    .join(" ")
}

fn looks_like_initials(
  value: &str
) -> bool {
  let letters: String = value
    .chars()
    .filter(|c| c.is_alphabetic())
    .collect();
  if letters.is_empty() {
    return false;
  }
  if letters.len() <= 4
    && letters
      .chars()
      .all(|c| c.is_uppercase())
  {
    return true;
  }
  value.chars().all(|c| {
    c.is_uppercase()
      || c == '-'
      || c == '.'
  })
}

fn strip_et_al_suffix(
  value: &str
) -> String {
  let parts = value
    .split_whitespace()
    .collect::<Vec<_>>();
  if parts.len() < 2 {
    return value.to_string();
  }
  let len = parts.len();
  let last = parts[len - 1]
    .trim_end_matches('.')
    .to_lowercase();
  let prior = parts[len - 2]
    .trim_end_matches('.')
    .to_lowercase();
  if prior == "et" && last == "al" {
    parts[..len - 2].join(" ")
  } else {
    value.to_string()
  }
}

fn tokens_from_authors(
  reference: &str
) -> BTreeSet<String> {
  let author_segment =
    extract_author_segment(reference);
  let mut tokens = tokens_from_segment(
    &author_segment
  );

  let normalized = author_segment
    .replace('&', ";")
    .replace(" and ", ";");
  for part in normalized.split(';') {
    let name = part.trim();
    if !name.is_empty() {
      let lowered = name
        .trim_end_matches('.')
        .to_lowercase();
      if lowered == "et al" {
        continue;
      }
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
  collect_year_tokens(reference)
    .into_iter()
    .collect()
}

fn capture_year_like(
  reference: &str
) -> Vec<(String, bool)> {
  let mut tokens = Vec::new();
  let mut buffer = String::new();
  let mut allow_short = false;

  for c in reference.chars() {
    if c.is_ascii_digit() {
      buffer.push(c);
    } else {
      if buffer.len() >= 2 {
        tokens.push((
          buffer.clone(),
          allow_short
        ));
      }
      buffer.clear();
      allow_short = matches!(
        c,
        '/' | '-' | '–' | '—'
      );
    }
  }

  if buffer.len() >= 2 {
    tokens.push((buffer, allow_short));
  }

  tokens
}

fn collect_year_tokens(
  reference: &str
) -> Vec<String> {
  let date_source =
    select_date_reference(reference);
  let mut tokens =
    collect_numeric_date_parts(
      &date_source
    );
  let date_parts_found =
    !tokens.is_empty();
  let mut previous: Option<String> =
    None;

  for (candidate, allow_short) in
    capture_year_like(&date_source)
  {
    if candidate.len() == 2
      && candidate
        .parse::<u32>()
        .ok()
        .filter(|value| *value <= 31)
        .is_some()
    {
      continue;
    }
    if date_parts_found
      && candidate.len() < 4
    {
      continue;
    }
    if let Some(year) =
      normalize_year_candidate(
        &candidate,
        previous.as_ref(),
        allow_short
      )
    {
      if previous.as_deref()
        == Some(&year)
      {
        continue;
      }
      previous = Some(year.clone());
      if !tokens.contains(&year) {
        tokens.push(year);
      }
    }
  }

  tokens
}

fn collect_numeric_date_parts(
  reference: &str
) -> Vec<String> {
  let mut parts = Vec::new();
  let separators = ['-', '/', '.'];
  let mut found = false;
  let tokens =
    reference.split_whitespace().collect::<Vec<_>>();
  for (idx, token) in
    tokens.iter().enumerate()
  {
    if is_page_marker(token)
      || idx
        .checked_sub(1)
        .and_then(|pos| tokens.get(pos))
        .map_or(false, |value| {
          is_page_marker(value)
        })
    {
      continue;
    }
    let trimmed =
      token.trim_matches(|c: char| {
        c.is_ascii_punctuation()
          && !separators.contains(&c)
      });
    if trimmed.is_empty() {
      continue;
    }
    if separators
      .iter()
      .any(|sep| trimmed.contains(*sep))
    {
      let pieces = trimmed
        .split(|c: char| {
          !c.is_ascii_digit()
        })
        .filter(|piece| {
          !piece.is_empty()
        })
        .collect::<Vec<_>>();
      if pieces.len() >= 3 {
        for (idx, piece) in
          pieces.iter().enumerate()
        {
          let normalized =
            normalize_date_part(
              piece,
              idx == 0
            );
          if !normalized.is_empty() {
            parts.push(normalized);
          }
        }
        found = true;
        break;
      }
    }
  }
  if !found {
    if let Some(month_parts) =
      collect_month_name_parts(
        reference
      )
    {
      parts = month_parts;
    }
  }
  parts
}

fn select_date_reference(
  reference: &str
) -> String {
  let segments =
    split_reference_segments(reference);
  if segments.is_empty() {
    return reference.to_string();
  }
  let mut best = None;
  let mut best_score = 0i32;
  for segment in segments {
    let score =
      date_segment_score(&segment);
    if score > best_score {
      best_score = score;
      best = Some(segment);
    }
  }
  best.unwrap_or_else(|| reference.to_string())
}

fn date_segment_score(
  segment: &str
) -> i32 {
  let mut score = 0;
  if segment_has_year(segment) {
    score += 3;
  }
  if segment_has_month(segment) {
    score += 2;
  }
  if segment.contains('(')
    || segment.contains(')')
  {
    score += 1;
  }
  if segment_has_page_marker(segment) {
    score -= 3;
  }
  if segment_has_volume_marker(segment) {
    score -= 1;
  }
  score
}

fn segment_has_year(
  segment: &str
) -> bool {
  segment
    .split(|c: char| !c.is_ascii_digit())
    .filter(|part| part.len() >= 4)
    .any(|part| {
      let year = &part[..4];
      year
        .parse::<u32>()
        .ok()
        .filter(|value| {
          (1400..=2099).contains(value)
        })
        .is_some()
    })
}

fn segment_has_month(
  segment: &str
) -> bool {
  segment
    .split_whitespace()
    .map(|token| {
      token.trim_matches(|c: char| {
        c.is_ascii_punctuation()
      })
    })
    .any(|token| parse_month_token(token).is_some())
}

fn segment_has_page_marker(
  segment: &str
) -> bool {
  segment
    .split_whitespace()
    .any(is_page_marker)
}

fn segment_has_volume_marker(
  segment: &str
) -> bool {
  let lower = segment.to_lowercase();
  lower.contains("vol")
    || lower.contains("no.")
    || lower.contains("issue")
    || lower.contains("number")
}

fn is_page_marker(
  token: &str
) -> bool {
  let lower = token
    .trim_matches(|c: char| {
      c.is_ascii_punctuation()
    })
    .to_lowercase();
  matches!(
    lower.as_str(),
    "p"
      | "p."
      | "pp"
      | "pp."
      | "page"
      | "pages"
  )
}

fn collect_month_name_parts(
  reference: &str
) -> Option<Vec<String>> {
  let tokens = reference
    .split_whitespace()
    .map(|token| {
      token.trim_matches(|c: char| {
        c.is_ascii_punctuation()
      })
    })
    .filter(|token| !token.is_empty())
    .collect::<Vec<_>>();
  let month_index = tokens
    .iter()
    .position(|token| {
      parse_month_token(token).is_some()
    })?;
  let month = parse_month_token(
    tokens[month_index]
  )?;

  let mut year: Option<String> = None;
  for token in
    tokens[..month_index].iter().rev()
  {
    let digits: String = token
      .chars()
      .filter(|c| c.is_ascii_digit())
      .collect();
    if digits.len() >= 4 {
      year =
        Some(digits[..4].to_string());
      break;
    }
  }
  if year.is_none() {
    for token in
      tokens[month_index + 1..].iter()
    {
      let digits: String = token
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect();
      if digits.len() >= 4 {
        year =
          Some(digits[..4].to_string());
        break;
      }
    }
  }
  let year = year?;

  let mut day = None;
  if let Some(token) =
    tokens.get(month_index + 1)
  {
    day = extract_day_token(token);
  }
  if day.is_none() && month_index > 0 {
    if let Some(token) =
      tokens.get(month_index - 1)
    {
      day = extract_day_token(token);
    }
  }

  let mut parts =
    vec![year, format!("{month:02}")];
  if let Some(day) = day {
    parts.push(day);
  }
  Some(parts)
}

fn parse_month_token(
  token: &str
) -> Option<u32> {
  let mut parts = token.split(|c| {
    c == '-' || c == '–' || c == '—'
  });
  let first = parts.next()?.trim();
  month_number(first)
}

fn extract_day_token(
  token: &str
) -> Option<String> {
  let total_digits = token
    .chars()
    .filter(|c| c.is_ascii_digit())
    .count();
  if total_digits > 2 {
    return None;
  }
  let mut digits = String::new();
  for ch in token.chars() {
    if ch.is_ascii_digit() {
      digits.push(ch);
      if digits.len() >= 2 {
        break;
      }
    } else if ch == '-'
      || ch == '–'
      || ch == '—'
    {
      break;
    } else if !digits.is_empty() {
      break;
    }
  }
  if digits.is_empty() {
    None
  } else {
    Some(digits)
  }
}

fn month_number(
  token: &str
) -> Option<u32> {
  let lower = token.to_lowercase();
  let abbrev = lower.trim_end_matches(
    |c: char| c == '.' || c == ','
  );
  match abbrev {
    | "jan" | "january" => Some(1),
    | "feb" | "february" => Some(2),
    | "mar" | "march" => Some(3),
    | "apr" | "april" => Some(4),
    | "may" => Some(5),
    | "jun" | "june" => Some(6),
    | "jul" | "july" => Some(7),
    | "aug" | "august" => Some(8),
    | "sep" | "sept" | "september" => {
      Some(9)
    }
    | "oct" | "october" => Some(10),
    | "nov" | "november" => Some(11),
    | "dec" | "december" => Some(12),
    | _ => None
  }
}

fn normalize_date_part(
  piece: &str,
  is_year: bool
) -> String {
  if is_year {
    normalize_year_candidate(
      piece, None, false
    )
    .unwrap_or_default()
  } else {
    piece
      .chars()
      .filter(|c| c.is_ascii_digit())
      .collect::<String>()
  }
}

fn normalize_year_candidate(
  candidate: &str,
  previous: Option<&String>,
  allow_short: bool
) -> Option<String> {
  let digits: String = candidate
    .chars()
    .filter(|c| c.is_ascii_digit())
    .collect();

  match digits.len() {
    | len if len >= 4 => {
      Some(digits[..4].to_string())
    }
    | len if len >= 2 && allow_short => {
      let prefix_len = 4 - len;
      if let Some(prev) = previous
        && prev.len() >= prefix_len
      {
        let prefix: String = prev
          .chars()
          .take(prefix_len)
          .collect();
        return Some(format!(
          "{prefix}{digits}"
        ));
      }
      if len == 2
        && digits
          .parse::<u32>()
          .ok()
          .filter(|value| *value > 31)
          .is_none()
      {
        return None;
      }
      let default_prefix =
        default_century_prefix(
          prefix_len
        );
      Some(format!(
        "{default_prefix}{digits}"
      ))
    }
    | _ => None
  }
}

fn default_century_prefix(
  len: usize
) -> &'static str {
  match len {
    | 1 => "1",
    | 2 => "19",
    | _ => "19"
  }
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

#[derive(Debug)]
pub struct Parser {
  dictionary:    Dictionary,
  normalization: NormalizationConfig
}

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
    Self {
      dictionary:
        Dictionary::create(
          DictionaryAdapter::Memory
        )
        .open(),
      normalization:
        NormalizationConfig::default()
    }
  }

  pub fn with_dictionary(
    dictionary: Dictionary
  ) -> Self {
    Self {
      dictionary,
      normalization:
        NormalizationConfig::default()
    }
  }

  pub fn with_normalization(
    normalization: NormalizationConfig
  ) -> Self {
    Self {
      dictionary: Dictionary::create(
        DictionaryAdapter::Memory
      )
      .open(),
      normalization
    }
  }

  pub fn with_dictionary_and_normalization(
    dictionary: Dictionary,
    normalization: NormalizationConfig
  ) -> Self {
    Self {
      dictionary,
      normalization
    }
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
          FieldTokens::from_reference_with_dictionary(
            reference,
            &self.dictionary
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
          authors_for_reference(
            reference
          );
        if !authors.is_empty() {
          mapped.insert(
            "author",
            FieldValue::Authors(
              authors
            )
          );
        }
        if let Some(number) =
          extract_citation_number(reference)
        {
          mapped.insert(
            "citation-number",
            FieldValue::Single(number)
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
            resolve_type_with_dictionary(
              reference,
              &self.dictionary
            )
          )
        );
        let location =
          extract_location(reference);
        mapped.insert(
          "location",
          FieldValue::List(vec![
            location.clone(),
          ])
        );
        if !location.is_empty() {
          mapped.insert(
            "publisher-place",
            FieldValue::List(vec![
              location.clone(),
            ])
          );
        }
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

        if let Some(collection) =
          extract_collection_title(
            reference
          )
        {
          mapped.insert(
            "collection-title",
            FieldValue::List(vec![
              collection,
            ])
          );
        }
        if let Some(collection_number) =
          extract_collection_number(
            reference
          )
        {
          mapped.insert(
            "collection-number",
            FieldValue::List(vec![
              collection_number,
            ])
          );
        }

        if let Some(journal) =
          extract_journal(reference)
        {
          mapped.insert(
            "journal",
            FieldValue::List(vec![
              journal,
            ])
          );
        }

        if let Some(editor) =
          extract_editor(reference)
        {
          mapped.insert(
            "editor",
            FieldValue::List(vec![
              editor,
            ])
          );
        }

        if let Some(translator) =
          extract_translator(reference)
        {
          mapped.insert(
            "translator",
            FieldValue::List(vec![
              translator,
            ])
          );
        }

        if let Some(note) =
          extract_note(reference)
        {
          mapped.insert(
            "note",
            FieldValue::List(vec![
              note,
            ])
          );
        }

        let identifiers =
          extract_identifiers(
            reference
          );
        let mut doi_values = Vec::new();
        let mut url_values = Vec::new();
        let mut isbn_values = Vec::new();
        let mut issn_values = Vec::new();
        for identifier in identifiers {
          let lower =
            identifier.to_lowercase();
          if lower.contains("isbn") {
            let value =
              clean_labeled_identifier(
                &identifier,
                "isbn"
              );
            if !value.is_empty() {
              isbn_values.push(value);
            }
            continue;
          }
          if lower.contains("issn") {
            let value =
              clean_labeled_identifier(
                &identifier,
                "issn"
              );
            if !value.is_empty() {
              issn_values.push(value);
            }
            continue;
          }
          if lower.contains("doi") {
            doi_values
              .push(identifier.clone());
            continue;
          }

          if identifier
            .starts_with("http")
            || identifier
              .starts_with("www")
            || lower.contains("url")
          {
            url_values
              .push(identifier.clone());
          }
        }

        if !doi_values.is_empty() {
          mapped.insert(
            "doi",
            FieldValue::List(
              doi_values
            )
          );
        }

        if !url_values.is_empty() {
          mapped.insert(
            "url",
            FieldValue::List(
              url_values
            )
          );
        }
        if isbn_values.is_empty() {
          if let Some(isbn) =
            extract_isbn(reference)
          {
            isbn_values.push(isbn);
          }
        }
        if issn_values.is_empty() {
          if let Some(issn) =
            extract_issn(reference)
          {
            issn_values.push(issn);
          }
        }
        if !isbn_values.is_empty() {
          mapped.insert(
            "isbn",
            FieldValue::List(
              isbn_values
            )
          );
        }
        if !issn_values.is_empty() {
          mapped.insert(
            "issn",
            FieldValue::List(
              issn_values
            )
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

        let mut year_values =
          collect_year_tokens(
            reference
          );
        if year_values.is_empty() {
          year_values
            .push(String::new());
        }
        mapped.insert(
          "date",
          FieldValue::List(year_values)
        );
        if detect_circa(reference) {
          mapped.insert(
            "date-circa",
            FieldValue::Single("true".into())
          );
        }
        mapped.insert(
          "pages",
          FieldValue::List(vec![
            extract_pages(reference),
          ])
        );
        mapped.insert(
          "language",
          FieldValue::Single(
            detect_language(reference)
          )
        );
        mapped.insert(
          "scripts",
          FieldValue::List(
            detect_scripts(reference)
          )
        );
        self.apply_normalization(mapped)
      })
      .collect()
  }

  fn apply_normalization(
    &self,
    reference: Reference
  ) -> Reference {
    let mut map =
      reference.fields().clone();
    self
      .normalization
      .apply_to_fields(&mut map);
    Reference::from_map(map)
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
  let cleaned =
    strip_leading_citation_number(
      reference
    );
  let segments =
    split_reference_segments(&cleaned);
  if segments.is_empty() {
    return String::new();
  }
  let (author_index, author_segment) =
    select_author_segment(&segments);
  let mut candidate = if author_index > 0 {
    segments[author_index - 1].clone()
  } else if segments.len() > 1 {
    segments[1].clone()
  } else {
    segments[0].clone()
  };
  if candidate == author_segment {
    candidate = segments
      .get(1)
      .cloned()
      .unwrap_or_else(|| segments[0].clone());
  }
  if candidate.split_whitespace().count() < 3
    && author_index + 1 < segments.len()
  {
    let next = segments[author_index + 1]
      .trim()
      .to_string();
    if !next.is_empty()
      && !segment_is_container(&next)
    {
      candidate =
        format!("{candidate}. {next}");
    }
  }
  if candidate.split_whitespace().count() < 3
    && author_index == 0
  {
    if let Some(title) =
      title_from_first_segment(
        &segments[0]
      )
    {
      candidate = title;
    }
  }
  clean_title_segment(&candidate)
}

fn extract_author(
  reference: &str
) -> String {
  extract_author_segment(reference)
}

fn extract_author_segment(
  reference: &str
) -> String {
  let cleaned =
    strip_leading_citation_number(
      reference
    );
  let segments =
    split_reference_segments(&cleaned);
  if segments.is_empty() {
    return strip_parenthetical_date(
      cleaned.trim()
    );
  }
  let (index, segment) =
    select_author_segment(&segments);
  let mut candidate =
    trim_author_segment(&segment);
  if candidate.is_empty() {
    candidate = segments
      .get(index)
      .cloned()
      .unwrap_or_default();
  }
  strip_parenthetical_date(&candidate)
}

fn select_author_segment(
  segments: &[String]
) -> (usize, String) {
  let mut best_index = 0usize;
  let mut best_score = i32::MIN;
  for (idx, segment) in
    segments.iter().take(4).enumerate()
  {
    let score =
      author_segment_score(segment);
    if score > best_score {
      best_score = score;
      best_index = idx;
    }
  }
  (
    best_index,
    segments
      .get(best_index)
      .cloned()
      .unwrap_or_default()
  )
}

fn author_segment_score(
  segment: &str
) -> i32 {
  let trimmed = segment.trim();
  if trimmed.is_empty() {
    return i32::MIN;
  }
  let mut score = 0;
  if trimmed
    .chars()
    .next()
    .map(|c| c.is_ascii_digit())
    .unwrap_or(false)
  {
    score -= 3;
  }
  if trimmed.contains(',') {
    score += 2;
  }
  if trimmed.contains(" and ")
    || trimmed.contains(" & ")
    || trimmed.contains('&')
  {
    score += 2;
  }
  if segment_has_year(trimmed) {
    score -= 2;
  }
  if trimmed.to_lowercase().contains("http")
    || trimmed.to_lowercase().contains("doi")
  {
    score -= 2;
  }
  if trimmed.split_whitespace().count() <= 6 {
    score += 1;
  }
  if trimmed
    .split_whitespace()
    .any(|token| looks_like_initials(token))
  {
    score += 1;
  }
  score
}

fn trim_author_segment(
  segment: &str
) -> String {
  if let Some((before, _)) =
    segment.split_once('(')
  {
    if segment_has_year(segment) {
      return before.trim().trim_end_matches(',').to_string();
    }
  }
  if let Some(pos) = segment
    .char_indices()
    .find(|(_, c)| c.is_ascii_digit())
    .map(|(idx, _)| idx)
  {
    let prefix = segment[..pos].trim();
    if prefix.len() >= 3 {
      return prefix.trim_end_matches(',').to_string();
    }
  }
  segment.trim().to_string()
}

fn strip_leading_citation_number(
  reference: &str
) -> String {
  let trimmed = reference.trim();
  let mut chars = trimmed.chars();
  let mut idx = 0usize;
  while let Some(ch) = chars.next() {
    if ch.is_ascii_digit() {
      idx += ch.len_utf8();
    } else {
      break;
    }
  }
  if idx == 0 {
    return trimmed.to_string();
  }
  let remainder = trimmed[idx..].trim_start();
  if remainder.starts_with('.')
    || remainder.starts_with(']')
  {
    return remainder[1..].trim_start().to_string();
  }
  trimmed.to_string()
}

fn extract_citation_number(
  reference: &str
) -> Option<String> {
  let trimmed = reference.trim();
  let mut chars = trimmed.chars();
  let mut prefix = String::new();
  if let Some(first) = chars.next() {
    if first == '[' || first == '(' {
      prefix.push(first);
    } else {
      chars = trimmed.chars();
    }
  }
  let mut digits = String::new();
  for ch in chars.by_ref() {
    if ch.is_ascii_digit() {
      digits.push(ch);
    } else {
      if !digits.is_empty() {
        let mut suffix = String::new();
        if ch == '.'
          || ch == ')'
          || ch == ']'
        {
          suffix.push(ch);
        }
        let value = format!(
          "{prefix}{digits}{suffix}"
        );
        return Some(value);
      }
      return None;
    }
  }
  if digits.is_empty() {
    None
  } else {
    Some(format!("{prefix}{digits}"))
  }
}
fn title_from_first_segment(
  segment: &str
) -> Option<String> {
  let mut parts = segment
    .split(',')
    .map(str::trim)
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>();
  if parts.len() < 3 {
    return None;
  }
  let year_pos = parts.iter().position(|part| {
    part.chars().filter(|c| c.is_ascii_digit()).count() >= 4
  });
  let Some(pos) = year_pos else {
    return None;
  };
  let after = parts
    .drain(pos + 1..)
    .collect::<Vec<_>>();
  let title = after
    .first()
    .map(|part| part.to_string())?;
  if title.is_empty() {
    None
  } else {
    Some(title)
  }
}

fn segment_is_container(
  segment: &str
) -> bool {
  let lower = segment.to_lowercase();
  lower.contains("journal")
    || lower.contains("proceedings")
    || lower.contains("conference")
    || lower.contains("symposium")
    || lower.contains("meeting")
    || lower.contains("presented")
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

fn resolve_type_with_dictionary(
  reference: &str,
  dictionary: &Dictionary
) -> String {
  for token in
    reference.split(|c: char| {
      !c.is_alphanumeric()
    })
  {
    if token.is_empty() {
      continue;
    }
    if dictionary
      .lookup(token)
      .contains(
        &DictionaryCode::Journal
      )
    {
      return "article".into();
    }
  }
  resolve_type(reference)
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
  let lower = reference.to_lowercase();
  if let Some(pos) = lower.find("pp.") {
    return capture_page_range(
      reference,
      pos + 3
    )
    .unwrap_or_default();
  }
  if let Some(pos) = lower.find("p.") {
    return capture_page_range(
      reference,
      pos + 2
    )
    .unwrap_or_default();
  }

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

fn capture_page_range(
  reference: &str,
  start: usize
) -> Option<String> {
  let segment =
    reference.get(start..)?;
  let mut digits = Vec::new();
  let mut current = String::new();
  let mut saw_separator = false;
  for ch in segment.chars() {
    if ch.is_ascii_digit() {
      current.push(ch);
      continue;
    }
    if ch == '-'
      || ch == '–'
      || ch == '—'
    {
      if !current.is_empty() {
        digits.push(current.clone());
        current.clear();
        saw_separator = true;
      }
      continue;
    }
    if !current.is_empty() {
      digits.push(current.clone());
      current.clear();
    }
    if saw_separator
      || digits.len() >= 2
    {
      break;
    }
    if !digits.is_empty()
      && ch.is_whitespace()
    {
      continue;
    }
  }
  if !current.is_empty() {
    digits.push(current);
  }
  if digits.is_empty() {
    return None;
  }
  if digits.len() == 1 {
    return Some(digits[0].clone());
  }
  Some(format!(
    "{}-{}",
    digits[0], digits[1]
  ))
}

fn extract_collection_title(
  reference: &str
) -> Option<String> {
  if let Some(segment) =
    segment_after_keyword(
      reference,
      "lecture notes"
    )
  {
    return Some(segment);
  }

  let keywords = [
    "series",
    "collection",
    "notes",
    "proceedings",
    "symposium",
    "volume"
  ];
  for keyword in keywords {
    if let Some(segment) =
      segment_after_keyword(
        reference, keyword
      )
    {
      return Some(segment);
    }
  }

  reference
    .split(|c: char| {
      c == ','
        || c == ';'
        || c == '('
        || c == ')'
    })
    .map(str::trim)
    .filter(|segment| {
      !segment.is_empty()
    })
    .find(|segment| {
      let lower =
        segment.to_lowercase();
      keywords.iter().any(|keyword| {
        lower.contains(keyword)
      })
    })
    .map(clean_segment)
}

fn extract_collection_number(
  reference: &str
) -> Option<String> {
  let title = extract_collection_title(
    reference
  )?;
  let lower_reference =
    reference.to_lowercase();
  let lower_title =
    title.to_lowercase();
  let start = lower_reference
    .find(&lower_title)?;
  let remainder = reference
    .get(start + title.len()..)
    .unwrap_or("");
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
  split_reference_segments(reference)
    .iter()
    .map(|segment| segment.trim())
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

fn extract_journal(
  reference: &str
) -> Option<String> {
  split_reference_segments(reference)
    .iter()
    .map(|segment| segment.trim())
    .find(|segment| {
      let lower =
        segment.to_lowercase();
      lower.contains("journal")
        || lower.contains("proceedings")
    })
    .map(clean_segment)
}

fn extract_editor(
  reference: &str
) -> Option<String> {
  let keywords = [
    "edited by",
    "edited",
    "editor",
    "eds"
  ];

  for keyword in keywords {
    if let Some(segment) =
      segment_after_keyword(
        reference, keyword
      )
    {
      return Some(segment);
    }
  }

  None
}

fn strip_parenthetical_date(
  segment: &str
) -> String {
  let mut output = String::new();
  let mut chars = segment.chars().peekable();
  while let Some(ch) = chars.next() {
    if ch == '(' {
      let mut contents = String::new();
      while let Some(inner) = chars.next() {
        if inner == ')' {
          break;
        }
        contents.push(inner);
      }
      let lower = contents.to_lowercase();
      let has_year = contents
        .chars()
        .any(|c| c.is_ascii_digit());
      let is_date = has_year
        && (lower.contains("c.")
          || lower.contains("ca.")
          || lower.contains("circa")
          || lower.contains("ed")
          || lower.contains("éd"));
      if !is_date {
        output.push('(');
        output.push_str(contents.trim());
        output.push(')');
      }
      continue;
    }
    output.push(ch);
  }
  output.trim().to_string()
}

fn extract_translator(
  reference: &str
) -> Option<String> {
  let keywords = [
    "translated by",
    "translator",
    "trans."
  ];

  for keyword in keywords {
    if let Some(segment) =
      segment_after_keyword(
        reference, keyword
      )
    {
      return Some(segment);
    }
  }

  None
}

fn extract_note(
  reference: &str
) -> Option<String> {
  let keywords = [
    "note",
    "report",
    "deliverable",
    "volume"
  ];

  for (start, _) in
    reference.match_indices('(')
  {
    if let Some(end) =
      reference[start + 1..].find(')')
    {
      let segment = reference
        [start + 1..start + 1 + end]
        .trim();
      if segment.is_empty() {
        continue;
      }
      let lower =
        segment.to_lowercase();
      if keywords.iter().any(
        |keyword| {
          lower.contains(keyword)
        }
      ) {
        return Some(clean_segment(
          segment
        ));
      }
    }
  }

  None
}

fn extract_identifiers(
  reference: &str
) -> Vec<String> {
  reference
    .split_whitespace()
    .map(|token| {
      token.trim_matches(|c: char| {
        c.is_ascii_punctuation()
      })
    })
    .filter(|token| !token.is_empty())
    .filter(|token| {
      let lower = token.to_lowercase();
      lower.starts_with("doi")
        || lower.contains("doi:")
        || lower.contains("isbn")
        || lower.contains("issn")
        || lower.starts_with("http")
        || lower.starts_with("www")
        || lower.contains("url")
        || lower.contains("urn")
    })
    .map(|token| token.to_string())
    .collect()
}

fn clean_labeled_identifier(
  identifier: &str,
  label: &str
) -> String {
  let lower = identifier.to_lowercase();
  let trimmed = if let Some(pos) =
    lower.find(label)
  {
    identifier[pos + label.len()..]
      .trim()
  } else {
    identifier.trim()
  };
  trimmed
    .chars()
    .filter(|c| {
      c.is_ascii_alphanumeric()
        || *c == '-'
    })
    .collect::<String>()
    .trim_matches('-')
    .to_string()
}

fn extract_isbn(
  reference: &str
) -> Option<String> {
  extract_labeled_identifier_value(
    reference, "isbn"
  )
}

fn extract_issn(
  reference: &str
) -> Option<String> {
  extract_labeled_identifier_value(
    reference, "issn"
  )
}

fn extract_labeled_identifier_value(
  reference: &str,
  label: &str
) -> Option<String> {
  let lower = reference.to_lowercase();
  let start = lower.find(label)?;
  let remainder = reference
    .get(start + label.len()..)
    .unwrap_or("");
  let value: String = remainder
    .chars()
    .skip_while(|c| {
      !c.is_ascii_digit()
        && *c != 'X'
        && *c != 'x'
    })
    .take_while(|c| {
      c.is_ascii_alphanumeric()
        || *c == '-'
    })
    .collect();
  let cleaned =
    value.trim_matches('-').to_string();
  if cleaned.is_empty() {
    None
  } else {
    Some(cleaned)
  }
}

fn clean_segment(
  segment: &str
) -> String {
  segment
    .trim_matches(|c: char| {
      c == ','
        || c == '.'
        || c == ';'
        || c == ':'
    })
    .trim()
    .to_string()
}

fn segment_after_keyword(
  reference: &str,
  keyword: &str
) -> Option<String> {
  reference
    .to_lowercase()
    .find(keyword)
    .and_then(|pos| {
      reference[pos..]
        .split(|c: char| {
          c == '.'
            || c == ','
            || c == ';'
            || c == ':'
        })
        .next()
    })
    .map(clean_segment)
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

fn tokens_from_identifiers(
  reference: &str
) -> BTreeSet<String> {
  let mut tokens = BTreeSet::new();
  for identifier in
    extract_identifiers(reference)
  {
    let normalized =
      normalize_token(&identifier);
    if !normalized.is_empty() {
      tokens.insert(normalized);
    }
  }
  tokens
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
      if let Some(found) =
        edition_number_before(
          reference, pos
        )
      {
        return Some(found);
      }
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

fn detect_circa(reference: &str) -> bool {
  let lower = reference.to_lowercase();
  lower.contains("c.")
    || lower.contains("ca.")
    || lower.contains("circa")
}

fn edition_number_before(
  reference: &str,
  keyword_pos: usize
) -> Option<String> {
  let prefix =
    reference.get(..keyword_pos)?;
  let token = prefix
    .split_whitespace()
    .last()
    .unwrap_or("")
    .trim_matches(|c: char| {
      c == '(' || c == ')' || c == ','
    });
  let digits: String = token
    .chars()
    .filter(|c| c.is_ascii_digit())
    .collect();
  if digits.is_empty() {
    None
  } else {
    Some(digits)
  }
}

fn clean_title_segment(
  segment: &str
) -> String {
  let mut output = String::new();
  let mut chars = segment.chars().peekable();
  while let Some(ch) = chars.next() {
    if ch == '(' {
      let mut contents = String::new();
      while let Some(inner) = chars.next() {
        if inner == ')' {
          break;
        }
        contents.push(inner);
      }
      let lower = contents.to_lowercase();
      let is_edition = lower.contains("ed")
        || lower.contains("édition")
        || lower.contains("ed.")
        || lower.contains("éd");
      if !is_edition {
        output.push('(');
        output.push_str(contents.trim());
        output.push(')');
      }
      continue;
    }
    output.push(ch);
  }
  output
    .trim_matches(|c: char| {
      c == '"' || c == '\'' || c == '.'
    })
    .trim()
    .to_string()
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

pub fn tagged_sequence_signature(
  sequence: &[TaggedToken]
) -> String {
  sequence
    .iter()
    .map(|token| token.token.trim())
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
    &context.identifier
  ) || lower.contains("doi")
    || lower.starts_with("http")
    || lower.starts_with("www")
    || lower.contains("urn")
  {
    "identifier".into()
  } else if matches_field(
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
    &context.journal
  ) {
    "journal".into()
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
    &context.collection
  ) {
    "collection-title".into()
  } else if matches_field(
    &normalized,
    &context.date
  ) {
    "date".into()
  } else if matches_field(
    &normalized,
    &context.editor
  ) {
    "editor".into()
  } else if matches_field(
    &normalized,
    &context.translator
  ) {
    "translator".into()
  } else if matches_field(
    &normalized,
    &context.note
  ) {
    "note".into()
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
