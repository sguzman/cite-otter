use std::collections::BTreeSet;

use crate::parser::types::TaggedToken;

use super::parse_month_token;

pub(crate) fn split_references(input: &str) -> Vec<String> {
  input
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty())
    .map(|line| line.to_string())
    .collect()
}

pub(crate) fn split_reference_segments(reference: &str) -> Vec<String> {
  let mut segments = Vec::new();
  let mut last_start = 0usize;
  let mut depth = 0usize;

  for (idx, ch) in reference.char_indices() {
    if ch == '(' {
      depth = depth.saturating_add(1);
    } else if ch == ')' && depth > 0 {
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
    let before = reference[..idx].trim_end();
    let mut token_start = 0usize;
    for (pos, ch) in before.char_indices() {
      if ch.is_whitespace() {
        token_start = pos + ch.len_utf8();
      }
    }
    let token = before[token_start..].trim();
    if !token.is_empty() && parse_month_token(token).is_some() {
      let mut next_chars = reference[idx + ch.len_utf8()..]
        .chars()
        .skip_while(|c| c.is_whitespace());
      if next_chars
        .next()
        .map(|next_char| next_char.is_ascii_digit())
        .unwrap_or(false)
      {
        continue;
      }
    }
    let mut next_chars = reference[idx + ch.len_utf8()..]
      .chars()
      .skip_while(|c| c.is_whitespace());
    let next = next_chars.next();
    let is_boundary = next.is_none_or(|next_char| {
      next_char.is_uppercase()
        || next_char.is_ascii_digit()
        || matches!(next_char, '"' | '“' | '‘')
    });
    if !is_boundary {
      continue;
    }
    let segment = reference[last_start..idx].trim().to_string();
    if !segment.is_empty() {
      segments.push(segment);
    }
    last_start = idx + ch.len_utf8();
  }

  let tail = reference[last_start..].trim().to_string();
  if !tail.is_empty() {
    segments.push(tail);
  }

  segments
}

fn is_initial_boundary(reference: &str, idx: usize) -> bool {
  let before = reference[..idx].trim_end();
  let mut token_start = 0usize;
  for (pos, ch) in before.char_indices() {
    if ch.is_whitespace() {
      token_start = pos + ch.len_utf8();
    }
  }
  let token = before[token_start..].trim();
  if token.len() != 1 || !token.chars().all(|c| c.is_alphabetic()) {
    return false;
  }
  let mut chars = reference[idx + 1..]
    .chars()
    .skip_while(|c| c.is_whitespace());
  let next = chars.next();
  let following = chars.next();
  if let (Some(next), Some(following)) = (next, following) {
    if next.is_ascii_digit() {
      return true;
    }
    if next == ';' || next == ',' {
      return true;
    }
    if next.is_alphabetic() && following.is_lowercase() {
      return true;
    }
  }
  matches!(
    (next, following),
    (Some(letter), Some('.')) if letter.is_alphabetic()
  )
}

pub(crate) fn tokens_from_segment(segment: &str) -> BTreeSet<String> {
  let mut tokens = BTreeSet::new();

  let normalized = normalize_token(segment);
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
      let normalized_part = normalize_token(part);
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

pub(crate) fn normalize_compare_value(value: &str) -> String {
  value
    .to_lowercase()
    .chars()
    .filter(|c| c.is_ascii_alphanumeric() || c.is_whitespace())
    .collect::<String>()
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ")
}

pub(crate) fn normalize_token(token: &str) -> String {
  let mut normalized = String::new();
  for ch in token.chars() {
    if ch.is_ascii_alphanumeric() {
      normalized.push(ch.to_ascii_lowercase());
    }
  }
  normalized
}

pub fn sequence_signature(tokens: &[String]) -> String {
  tokens
    .iter()
    .map(|token| token.trim())
    .filter(|token| !token.is_empty())
    .map(|token| token.to_string())
    .collect::<Vec<_>>()
    .join(" ")
}

pub fn tagged_sequence_signature(sequence: &[TaggedToken]) -> String {
  sequence
    .iter()
    .map(|token| token.token.trim())
    .filter(|token| !token.is_empty())
    .map(|token| token.to_string())
    .collect::<Vec<_>>()
    .join(" ")
}
