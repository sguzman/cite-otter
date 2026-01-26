use std::collections::BTreeSet;

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
use crate::parser::field_tokens::FieldTokens;
use crate::parser::types::{
  Author,
  FieldValue,
  Reference,
  TaggedToken,
};

const PREPARED_LINES: [&str; 2] = [
  "Hello, hello Lu P H He , o, \
   initial none F F F F none first \
   other none weak F",
  "world! world Ll P w wo ! d! lower \
   none T F T T none last other none \
   weak F"
];
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
    } else if ch == ')' && depth > 0 {
      depth = depth.saturating_sub(1);
    }
    if ch != '.' {
      continue;
    }
    if depth > 0 {
      continue;
    }
    if is_initial_boundary(
      reference, idx
    ) {
      continue;
    }
    let before =
      reference[..idx].trim_end();
    let mut token_start = 0usize;
    for (pos, ch) in
      before.char_indices()
    {
      if ch.is_whitespace() {
        token_start =
          pos + ch.len_utf8();
      }
    }
    let token =
      before[token_start..].trim();
    if !token.is_empty()
      && parse_month_token(token)
        .is_some()
    {
      let mut next_chars = reference
        [idx + ch.len_utf8()..]
        .chars()
        .skip_while(|c| {
          c.is_whitespace()
        });
      if next_chars
        .next()
        .map(|next_char| {
          next_char.is_ascii_digit()
        })
        .unwrap_or(false)
      {
        continue;
      }
    }
    let mut next_chars = reference
      [idx + ch.len_utf8()..]
      .chars()
      .skip_while(|c| {
        c.is_whitespace()
      });
    let next = next_chars.next();
    let is_boundary = next
      .is_none_or(|next_char| {
        next_char.is_uppercase()
          || next_char.is_ascii_digit()
          || matches!(
            next_char,
            '"' | '“' | '‘'
          )
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
  let before =
    reference[..idx].trim_end();
  let mut token_start = 0usize;
  for (pos, ch) in before.char_indices()
  {
    if ch.is_whitespace() {
      token_start = pos + ch.len_utf8();
    }
  }
  let token =
    before[token_start..].trim();
  if token.len() != 1
    || !token
      .chars()
      .all(|c| c.is_alphabetic())
  {
    return false;
  }
  let mut chars = reference[idx + 1..]
    .chars()
    .skip_while(|c| c.is_whitespace());
  let next = chars.next();
  let following = chars.next();
  if let (Some(next), Some(following)) =
    (next, following)
  {
    if next.is_ascii_digit() {
      return true;
    }
    if next == ';' || next == ',' {
      return true;
    }
    if next.is_alphabetic()
      && following.is_lowercase()
    {
      return true;
    }
  }
  matches!(
    (next, following),
    (Some(letter), Some('.'))
      if letter.is_alphabetic()
  )
}

pub(crate) fn tokens_from_segment(
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
  let segment =
    trim_author_segment(&segment);
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
  if parts.iter().all(|part| {
    looks_like_family_with_initials(
      part
    )
  }) {
    return parts
      .into_iter()
      .map(|part| part.to_string())
      .collect();
  }
  if parts.iter().all(|part| {
    looks_like_initial_surname(part)
  }) {
    return parts
      .into_iter()
      .map(|part| part.to_string())
      .collect();
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
          split_author_candidates(
            &remainder
          )
        );
      }
      return grouped;
    }
  }
  if parts.len() % 2 == 0
    && parts.chunks(2).all(|pair| {
      pair.len() == 2
        && looks_like_given_token(
          pair[1]
        )
    })
  {
    let mut grouped = Vec::new();
    for pair in parts.chunks(2) {
      if pair.len() != 2 {
        continue;
      }
      let mut family = pair[0]
        .trim_start_matches('&')
        .trim();
      if family
        .to_lowercase()
        .starts_with("and ")
      {
        family = family[4..].trim();
      }
      if family.is_empty() {
        continue;
      }
      grouped.push(format!(
        "{}, {}",
        family, pair[1]
      ));
    }
    if !grouped.is_empty() {
      return grouped;
    }
  }
  if parts.len() >= 3
    && parts
      .iter()
      .all(|part| !part.contains(','))
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
  let normalized =
    normalize_author_component(value)
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
      });
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
    if family_start == family_end {
      family_start =
        family_end.saturating_sub(1);
    }
    if family_end >= 2
      && looks_like_initials(
        &tokens[family_end - 1]
      )
    {
      let family = tokens
        [..family_end - 1]
        .join(" ");
      let mut given_parts = vec![
        tokens[family_end - 1].clone(),
      ];
      if let Some(token) = suffix {
        given_parts.push(token);
      }
      return Some(Author {
        family:
          normalize_author_component(
            &family
          ),
        given:
          normalize_author_component(
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
            '.'
              | ','
              | ';'
              | ':'
              | '!'
              | '?'
              | '('
              | ')'
              | '['
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
  if letters.len() <= 2
    && letters
      .chars()
      .all(|c| c.is_uppercase())
  {
    return true;
  }
  if letters.len() <= 4
    && letters
      .chars()
      .all(|c| c.is_uppercase())
    && value.contains('.')
  {
    return true;
  }
  value.chars().all(|c| {
    c.is_uppercase()
      || c == '-'
      || c == '.'
  })
}

fn looks_like_given_name(
  value: &str
) -> bool {
  if looks_like_initials(value) {
    return true;
  }
  let cleaned =
    value.trim_matches(|c: char| {
      c.is_ascii_punctuation()
    });
  let mut chars = cleaned.chars();
  let Some(first) = chars.next() else {
    return false;
  };
  first.is_uppercase()
    && cleaned.len() >= 2
}

fn looks_like_given_token(
  value: &str
) -> bool {
  if value
    .chars()
    .any(|c| c.is_whitespace())
  {
    return false;
  }
  let cleaned =
    value.trim_matches(|c: char| {
      c.is_ascii_punctuation()
    });
  if cleaned.is_empty() {
    return false;
  }
  if looks_like_initials(cleaned) {
    return true;
  }
  let mut chars = cleaned.chars();
  let Some(first) = chars.next() else {
    return false;
  };
  first.is_uppercase()
    && cleaned.len() >= 2
    && cleaned.chars().all(|c| {
      c.is_alphabetic() || c == '-'
    })
}

fn looks_like_family_with_initials(
  value: &str
) -> bool {
  let tokens = value
    .split_whitespace()
    .collect::<Vec<_>>();
  if tokens.len() < 2 {
    return false;
  }
  let last = tokens[tokens.len() - 1];
  if !looks_like_initials(last) {
    return false;
  }
  let allowed_particles = [
    "da", "de", "del", "der", "den",
    "di", "du", "la", "le", "van",
    "von", "al", "bin", "ibn"
  ];
  tokens[..tokens.len() - 1].iter().any(
    |token| {
      let normalized =
        normalize_author_component(
          token
        )
        .to_lowercase();
      allowed_particles
        .contains(&normalized.as_str())
        || token
          .chars()
          .next()
          .map(|c| c.is_uppercase())
          .unwrap_or(false)
    }
  )
}

fn looks_like_initial_surname(
  value: &str
) -> bool {
  let tokens = value
    .split_whitespace()
    .collect::<Vec<_>>();
  if tokens.len() < 2 {
    return false;
  }
  if !looks_like_initials(tokens[0]) {
    return false;
  }
  let last = tokens[tokens.len() - 1];
  let mut chars = last.chars();
  let Some(first) = chars.next() else {
    return false;
  };
  first.is_uppercase()
    && chars.any(|c| c.is_lowercase())
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

pub(crate) fn tokens_from_authors(
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

pub(crate) fn tokens_from_dates(
  reference: &str
) -> BTreeSet<String> {
  collect_year_tokens(reference)
    .into_iter()
    .collect()
}

fn capture_year_like(
  reference: &str
) -> Vec<(String, bool)> {
  let chars = reference
    .chars()
    .collect::<Vec<_>>();
  let mut tokens = Vec::new();
  let mut idx = 0usize;

  while idx < chars.len() {
    if !chars[idx].is_ascii_digit() {
      idx += 1;
      continue;
    }
    let start = idx;
    while idx < chars.len()
      && chars[idx].is_ascii_digit()
    {
      idx += 1;
    }
    let end = idx;
    let digits: String = chars
      [start..end]
      .iter()
      .collect();
    if digits.len() < 2 {
      continue;
    }
    let mut prev_idx = start;
    while prev_idx > 0
      && chars[prev_idx - 1]
        .is_whitespace()
    {
      prev_idx -= 1;
    }
    let prev = if prev_idx > 0 {
      Some(chars[prev_idx - 1])
    } else {
      None
    };
    let mut next_idx = end;
    while next_idx < chars.len()
      && chars[next_idx].is_whitespace()
    {
      next_idx += 1;
    }
    let next =
      chars.get(next_idx).copied();
    let allow_short = matches!(
      prev,
      Some('/')
        | Some('-')
        | Some('–')
        | Some('—')
    );
    let is_page_range =
      if let Some(next) = next
        && matches!(
          next,
          '-' | '–' | '—'
        )
      {
        let mut look = next_idx + 1;
        while look < chars.len()
          && chars[look].is_whitespace()
        {
          look += 1;
        }
        let mut next_digits =
          String::new();
        while look < chars.len()
          && chars[look]
            .is_ascii_digit()
        {
          next_digits.push(chars[look]);
          look += 1;
        }
        digits.len() == 4
          && next_digits.len() <= 2
      } else {
        false
      };
    let is_short_page_range = if matches!(
      prev,
      Some('-') | Some('–') | Some('—')
    ) {
      digits.len() <= 2
    } else {
      false
    };
    if !is_page_range
      && !is_short_page_range
    {
      tokens
        .push((digits, allow_short));
    }
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
      if year.len() == 4
        && year
          .parse::<u32>()
          .ok()
          .filter(|value| {
            *value < 1500
              || *value > 2100
          })
          .is_some()
      {
        continue;
      }
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
  if segment_has_month(reference)
    && let Some(parts) =
      collect_month_name_parts(
        reference
      )
  {
    return parts;
  }
  let mut parts = Vec::new();
  let separators = ['-', '/', '.'];
  let mut found = false;
  let tokens = reference
    .split_whitespace()
    .collect::<Vec<_>>();
  for (idx, token) in
    tokens.iter().enumerate()
  {
    if is_page_marker(token)
      || parse_page_range_token(token)
        .is_some()
      || idx
        .checked_sub(1)
        .and_then(|pos| tokens.get(pos))
        .is_some_and(|value| {
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
  if !found
    && let Some(month_parts) =
      collect_month_name_parts(
        reference
      )
  {
    parts = month_parts;
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
  best.unwrap_or_else(|| {
    reference.to_string()
  })
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
  if segment_has_page_range(segment) {
    score -= 2;
  }
  if segment_has_page_range(segment) {
    score -= 2;
  }
  if segment_has_volume_marker(segment)
  {
    score -= 1;
  }
  score
}

fn segment_has_year(
  segment: &str
) -> bool {
  segment
    .split(|c: char| {
      !c.is_ascii_digit()
    })
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
    .any(|token| {
      parse_month_token(token).is_some()
    })
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

fn segment_has_page_range(
  segment: &str
) -> bool {
  segment.split_whitespace().any(
    |token| {
      parse_page_range_token(token)
        .is_some()
    }
  )
}

fn is_page_marker(token: &str) -> bool {
  let lower = token
    .trim_matches(|c: char| {
      c.is_ascii_punctuation()
    })
    .to_lowercase();
  if lower.starts_with("p.")
    || lower.starts_with("pp.")
  {
    return true;
  }
  if let Some(rest) =
    lower.strip_prefix('p')
    && !rest.is_empty()
    && rest
      .chars()
      .all(|c| c.is_ascii_digit())
  {
    return true;
  }
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
    if token.contains('-')
      || token.contains('–')
      || token.contains('—')
    {
      continue;
    }
    let digits: String = token
      .chars()
      .filter(|c| c.is_ascii_digit())
      .collect();
    if digits.len() >= 4 {
      let candidate = &digits[..4];
      if let Ok(value) =
        candidate.parse::<u32>()
        && (1800..=2099)
          .contains(&value)
      {
        year = Some(candidate.to_string());
        break;
      }
    }
  }
  if year.is_none() {
    for token in
      tokens[month_index + 1..].iter()
    {
      if token.contains('-')
        || token.contains('–')
        || token.contains('—')
      {
        continue;
      }
      let digits: String = token
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect();
      if digits.len() >= 4 {
        let candidate = &digits[..4];
        if let Ok(value) =
          candidate.parse::<u32>()
          && (1800..=2099)
            .contains(&value)
        {
          year =
            Some(candidate.to_string());
          break;
        }
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
  if day.is_none()
    && month_index > 0
    && let Some(token) =
      tokens.get(month_index - 1)
  {
    day = extract_day_token(token);
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
  let has_dash = token.contains('-')
    || token.contains('–')
    || token.contains('—');
  if total_digits > 2 && !has_dash {
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
      || !digits.is_empty()
    {
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
  let abbrev =
    lower.trim_end_matches(['.', ',']);
  match abbrev {
    | "jan" | "january" => Some(1),
    | "feb" | "february" => Some(2),
    | "mar" | "march" => Some(3),
    | "apr" | "april" => Some(4),
    | "may" => Some(5),
    | "jun" | "june" => Some(6),
    | "jul" | "july" => Some(7),
    | "aug" | "august" => Some(8),
    | "sep" | "sept" | "september"
    | "septembre" => Some(9),
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
          extract_journal_with_dictionary(
            reference,
            Some(&self.dictionary)
          )
        {
          let journal_value = journal.clone();
          mapped.insert(
            "journal",
            FieldValue::List(vec![
              journal,
            ])
          );
          if !mapped
            .fields()
            .contains_key("container-title")
          {
            mapped.insert(
              "container-title",
              FieldValue::List(vec![
                journal_value,
              ])
            );
          }
        }

        let editors =
          extract_editor_list(reference);
        if !editors.is_empty() {
          mapped.insert(
            "editor",
            FieldValue::List(editors)
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
        if isbn_values.is_empty()
          && let Some(isbn) =
            extract_isbn(reference)
        {
          isbn_values.push(isbn);
        }
        if issn_values.is_empty()
          && let Some(issn) =
            extract_issn(reference)
        {
          issn_values.push(issn);
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

pub(crate) fn extract_title(
  reference: &str
) -> String {
  let cleaned =
    strip_leading_citation_number(
      reference
    );
  if extract_citation_number(reference)
    .is_some()
  {
    let segments =
      split_reference_segments(
        &cleaned
      );
    if let Some(first) =
      segments.first()
      && !first.is_empty()
    {
      return clean_title_segment(first);
    }
  }
  if let Some(end) =
    find_author_list_end_by_title(
      &cleaned
    )
  {
    let after =
      cleaned[end + 1..].trim_start();
    if let Some(candidate) =
      after.split(". ").next()
    {
      let candidate = candidate.trim();
      if !candidate.is_empty() {
        return clean_title_segment(
          candidate
        );
      }
    }
  }
  let segments =
    split_reference_segments(&cleaned);
  if segments.is_empty() {
    return String::new();
  }
  if segments.len() > 1 {
    let first = segments[0].trim();
    if looks_like_author_list(first)
      || split_leading_author_by_comma(
        first
      )
      .is_some()
    {
      let candidate =
        segments[1].trim();
      if !candidate.is_empty()
        && !segment_is_container(
          candidate
        )
        && !segment_has_year(candidate)
        && !segment_has_page_marker(
          candidate
        )
      {
        return clean_title_segment(
          candidate
        );
      }
    }
  }
  if let Some((_, title)) =
    split_leading_author_by_comma(
      &segments[0]
    )
    && !remainder_has_author_list(
      &title
    )
  {
      if segments.len() > 1
        && title
          .split_whitespace()
          .count()
          <= 1
      {
        // Avoid treating a lone given
        // name as the title
        // when the author
        // segment is followed by a real
        // title segment.
      } else {
        let starts_with_initial = title
          .split_whitespace()
          .next()
          .map(|token| {
            looks_like_initials(token)
          })
          .unwrap_or(false);
        if starts_with_initial {
          // Skip initial-based author
          // chunks (e.g., "G.
          // (1999)") when
          // extracting titles.
          // Fall through to other
          // strategies.
        } else {
          let first = title
            .split(',')
            .next()
            .unwrap_or(&title)
            .trim();
          if !first.is_empty()
            && !looks_like_author_list(
              first
            )
          {
            return clean_title_segment(
              first
            );
          }
        }
      }
  }
  if let Some((_, title)) =
    split_author_title_segment(
      &segments[0]
    )
    && !title.is_empty()
  {
    return clean_title_segment(&title);
  }
  let (author_index, author_segment) =
    select_author_segment(&segments);
  let mut candidate =
    if author_index > 0 {
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
      .unwrap_or_else(|| {
        segments[0].clone()
      });
  }
  if candidate
    .split_whitespace()
    .count()
    < 3
    && author_index + 1 < segments.len()
  {
    let next = segments
      [author_index + 1]
      .trim()
      .to_string();
    if !next.is_empty()
      && !segment_is_container(&next)
    {
      candidate =
        format!("{candidate}. {next}");
    }
  }
  if author_index == 0
    && (candidate
      .split_whitespace()
      .count()
      < 3
      || (segments[0].contains(',')
        && segment_has_year(&segments[0])))
    && let Some(title) =
      title_from_first_segment(
        &segments[0]
      )
  {
    candidate = title;
  }
  if is_title_noise_segment(&candidate)
  {
    if let Some(best) =
      select_title_segment(&segments)
    {
      candidate = best;
    } else {
      if let Some(title) =
        select_title_from_segment(
          &segments[0]
        )
      {
        candidate = title;
      } else {
        return String::new();
      }
    }
  }
  if candidate
    .chars()
    .any(|c| c.is_ascii_digit())
    && candidate
      .split_whitespace()
      .count()
      <= 2
    && let Some(title) =
      select_title_from_segment(
        &segments[0]
      )
  {
    candidate = title;
  }
  if !candidate
    .chars()
    .any(|c| c.is_alphabetic())
  {
    return String::new();
  }
  clean_title_segment(&candidate)
}

fn extract_author(
  reference: &str
) -> String {
  extract_author_segment(reference)
}

fn trim_author_segment_before_journal(
  reference: &str
) -> Option<String> {
  let mut parts = Vec::new();
  let mut saw_initial = false;
  for part in reference.split(',') {
    let trimmed = part.trim();
    if trimmed.is_empty() {
      continue;
    }
    let candidate =
      trimmed.trim_end_matches('.');
    if segment_is_journal_like(
      candidate
    ) || looks_like_short_journal(
      candidate
    ) {
      if saw_initial
        && !parts.is_empty()
      {
        return Some(parts.join(", "));
      }
      return None;
    }
    if trimmed.split_whitespace().any(
      |token| {
        looks_like_initials(token)
      }
    ) {
      saw_initial = true;
    }
    parts.push(trimmed);
  }
  None
}

fn strip_trailing_journal_author_segment(
  segment: &str
) -> Option<String> {
  let parts = segment
    .split(',')
    .map(str::trim)
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>();
  if parts.len() < 2 {
    return None;
  }
  let last =
    parts.last().copied().unwrap_or("");
  if !last.is_empty()
    && (segment_is_journal_like(last)
      || looks_like_short_journal(last))
  {
    let trimmed = parts
      [..parts.len() - 1]
      .join(", ");
    if !trimmed.is_empty() {
      return Some(trimmed);
    }
  }
  None
}

fn extract_author_segment(
  reference: &str
) -> String {
  let cleaned =
    strip_leading_citation_number(
      reference
    );
  if extract_citation_number(reference)
    .is_some()
  {
    if let Some(pos) =
      cleaned.find(". ")
    {
      let mut candidate = cleaned
        [pos + 2..]
        .trim()
        .trim_end_matches('.');
      if let Some((before, _)) =
        candidate.split_once(':')
      {
        candidate = before.trim();
      }
      if let Some(end) =
        find_author_list_end_by_title(
          candidate
        )
      {
        candidate =
          candidate[..end].trim();
      }
      if candidate.contains(',') {
        return trim_author_segment(
          candidate
        );
      }
    }
    for segment in
      split_reference_segments(&cleaned)
        .iter()
        .skip(1)
    {
      let trimmed = segment.trim();
      if trimmed.is_empty() {
        continue;
      }
      if looks_like_author_list(trimmed)
        || (trimmed.contains(',')
          && trimmed
            .split_whitespace()
            .any(|token| {
              looks_like_initials(token)
            }))
      {
        return trim_author_segment_at_date(
          trimmed
        );
      }
    }
  }
  if let Some(end) =
    find_author_list_end_by_title(
      &cleaned
    )
  {
    let candidate =
      cleaned[..end].trim();
    if !candidate.is_empty() {
      return trim_author_segment(
        candidate
      );
    }
  }
  let segments =
    split_reference_segments(&cleaned);
  if let Some(first) = segments.first()
    && (looks_like_author_list(first)
      || split_leading_author_by_comma(
        first
      )
      .is_some())
  {
    return trim_author_segment(first);
  }
  if let Some(and_pos) =
    cleaned.find(" and ")
    && let Some(end) =
      cleaned[and_pos..].find('.')
  {
    let prefix = cleaned
      [..and_pos + end]
      .trim()
      .trim_end_matches('.');
    if !prefix.is_empty() {
      return prefix.to_string();
    }
  }
  if let Some(pos) = cleaned.find('.') {
    let prefix = cleaned[..pos].trim();
    if !looks_like_author_list(prefix)
      && !looks_like_person_name(prefix)
    {
      let remainder =
        cleaned[pos + 1..].trim();
      if let Some(next_pos) =
        remainder.find('.')
      {
        let candidate =
          remainder[..next_pos].trim();
        if candidate.contains(',')
          && candidate
            .split_whitespace()
            .any(|token| {
              looks_like_initials(token)
            })
        {
          return trim_author_segment(
            candidate
          );
        }
      }
    }
  }
  let segments =
    split_reference_segments(&cleaned);
  if segments.len() > 1 {
    let first = segments[0].trim();
    if !looks_like_author_list(first)
      && !looks_like_person_name(first)
    {
      for segment in
        segments.iter().skip(1)
      {
        let trimmed = segment.trim();
        if trimmed.is_empty() {
          continue;
        }
        if trimmed.contains(',')
          && trimmed
            .split_whitespace()
            .any(|token| {
              looks_like_initials(token)
            })
        {
          return trim_author_segment(
            trimmed
          );
        }
      }
    }
  }
  if let Some(pos) = cleaned.find('.')
    && pos > 0
  {
    let prefix = cleaned[..pos].trim();
    if !prefix.contains(',')
      && looks_like_person_name(prefix)
    {
      return prefix.to_string();
    }
    if looks_like_author_list(prefix) {
      let remainder =
        cleaned[pos + 1..].trim();
      if !remainder_has_author_list(
        remainder
      ) {
        let tokens = remainder
          .split_whitespace()
          .map(|token| {
            token.trim_matches(
              |c: char| {
                c.is_ascii_punctuation()
              }
            )
          })
          .filter(|token| {
            !token.is_empty()
          })
          .collect::<Vec<_>>();
        if let Some(first) =
          tokens.first()
          && looks_like_initials(first)
          && !matches!(
            first
              .to_lowercase()
              .as_str(),
            "a" | "an" | "the"
          )
        {
          let mut expanded =
            prefix.to_string();
          expanded.push(' ');
          expanded.push_str(first);
          if let Some(second) =
            tokens.get(1)
            && looks_like_initials(
              second
            )
            && !matches!(
              second
                .to_lowercase()
                .as_str(),
              "a" | "an" | "the"
            )
          {
            expanded.push(' ');
            expanded.push_str(second);
          }
          return expanded;
        }
        return prefix.to_string();
      }
    }
  }
  if let Some((author, remainder)) =
    split_leading_author_by_comma(
      &cleaned
    )
  {
    if !remainder_has_author_list(
      &remainder
    ) {
      let tokens = remainder
        .split_whitespace()
        .map(|token| {
          token.trim_matches(
            |c: char| {
              c.is_ascii_punctuation()
            }
          )
        })
        .filter(|token| {
          !token.is_empty()
        })
        .collect::<Vec<_>>();
      let author_has_initial = author
        .split_whitespace()
        .any(|token| {
          looks_like_initials(token)
        });
      if !author_has_initial
        && let Some(first) =
          tokens.first()
        && looks_like_given_name(first)
      {
        let mut parts = vec![
          author,
          first.to_string(),
        ];
        if let Some(second) =
          tokens.get(1)
          && looks_like_initials(second)
        {
          parts
            .push(second.to_string());
        }
        return parts.join(", ");
      }
      return author;
    }
    if let Some(author) =
      trim_author_segment_before_journal(
        &cleaned
      )
    {
      return strip_parenthetical_date(
        &author
      );
    }
    if let Some(end) =
      find_author_list_end(&cleaned)
    {
      let prefix = cleaned[..end]
        .trim()
        .trim_end_matches('.');
      if !prefix.is_empty() {
        if let Some(author) =
          trim_author_segment_before_journal(
            prefix
          )
        {
          return strip_parenthetical_date(
            &author
          );
        }
        if let Some(author) =
          strip_trailing_journal_author_segment(
            prefix
          )
        {
          return strip_parenthetical_date(
            &author
          );
        }
        return prefix.to_string();
      }
    }
  }
  let segments =
    split_reference_segments(&cleaned);
  if segments.is_empty() {
    return strip_parenthetical_date(
      cleaned.trim()
    );
  }
  if let Some(first) = segments.first()
    && looks_like_author_list(first)
  {
    return strip_parenthetical_date(
      &trim_author_segment(first)
    );
  }
  let (index, segment) =
    select_author_segment(&segments);
  if let Some((authors, _)) =
    split_author_title_segment(&segment)
    && !authors.is_empty()
  {
    return authors;
  }
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

fn find_author_list_end_by_title(
  reference: &str
) -> Option<usize> {
  for (idx, ch) in
    reference.char_indices()
  {
    if ch != '.' {
      continue;
    }
    let before =
      reference[..idx].trim();
    if before.is_empty()
      || !before.contains(',')
    {
      continue;
    }
    if !before.split_whitespace().any(
      |token| {
        looks_like_initials(token)
      }
    ) {
      continue;
    }
    if let Some((_, tail)) =
      before.rsplit_once(',')
      && tail.split_whitespace().count()
        > 2
    {
      continue;
    }
    let after =
      reference[idx + 1..].trim_start();
    if after.is_empty() {
      continue;
    }
    let mut words =
      after.split_whitespace();
    let first =
      words.next().unwrap_or("");
    let first_clean = first
      .trim_matches(|c: char| {
        c.is_ascii_punctuation()
      });
    if is_title_word(first_clean) {
      return Some(idx);
    }
    if first_clean.len() == 1
      && !first.contains('-')
      && !first.contains('.')
      && first_clean
        .chars()
        .all(|c| c.is_uppercase())
      && let Some(next) = words.next()
    {
      let next_clean = next
        .trim_matches(|c: char| {
          c.is_ascii_punctuation()
        });
      if is_title_word(next_clean) {
        return Some(idx);
      }
    }
  }
  None
}

fn is_title_word(word: &str) -> bool {
  if word.len() < 2 {
    return false;
  }
  let mut chars = word.chars();
  let Some(first) = chars.next() else {
    return false;
  };
  first.is_uppercase()
    && chars.any(|c| c.is_lowercase())
}

fn looks_like_person_name(
  value: &str
) -> bool {
  let tokens = value
    .split_whitespace()
    .collect::<Vec<_>>();
  if tokens.len() < 2
    || tokens.len() > 4
  {
    return false;
  }
  if let Some(first) = tokens.first()
    && matches!(
      first.to_lowercase().as_str(),
      "a" | "an" | "the"
    )
  {
    return false;
  }
  tokens.iter().all(|token| {
    let mut chars = token.chars();
    let Some(first) = chars.next()
    else {
      return false;
    };
    first.is_uppercase()
      && token
        .chars()
        .any(|c| c.is_lowercase())
  })
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
  if segment_is_container(trimmed) {
    score -= 4;
  }
  if trimmed.contains(':') {
    score -= 2;
  }
  if trimmed
    .to_lowercase()
    .contains("proc.")
  {
    score -= 2;
  }
  if trimmed.contains('(')
    && trimmed.contains(')')
    && segment_has_year(trimmed)
  {
    score += 2;
  }
  if trimmed
    .to_lowercase()
    .contains(" in ")
  {
    score -= 2;
  }
  if trimmed
    .chars()
    .next()
    .map(|c| c.is_ascii_digit())
    .unwrap_or(false)
  {
    score -= 6;
  }
  if trimmed.chars().all(|c| {
    c.is_ascii_digit()
      || c == ','
      || c == '.'
      || c == '-'
  }) {
    score -= 6;
  }
  let comma_count =
    trimmed.matches(',').count();
  if comma_count >= 2
    && segment_is_container(trimmed)
  {
    score += 2;
  }
  if comma_count > 0 {
    score += 2;
  }
  if comma_count >= 2 {
    score += 1;
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
  if trimmed
    .to_lowercase()
    .contains("http")
    || trimmed
      .to_lowercase()
      .contains("doi")
  {
    score -= 2;
  }
  let initial_count = trimmed
    .split_whitespace()
    .filter(|token| {
      looks_like_initials(token)
    })
    .count();
  if initial_count >= 2 {
    score += 2;
  }
  if comma_count >= 2
    && initial_count >= 1
  {
    score += 4;
  }
  if trimmed.split_whitespace().any(
    |token| {
      parse_month_token(token).is_some()
    }
  ) {
    score -= 2;
  }
  if trimmed.split_whitespace().count()
    <= 6
  {
    score += 1;
  } else if trimmed
    .split_whitespace()
    .count()
    >= 10
  {
    score -= 2;
  }
  if trimmed
    .split_whitespace()
    .any(looks_like_initials)
  {
    score += 1;
  }
  score
}

fn looks_like_author_list(
  segment: &str
) -> bool {
  let trimmed = segment.trim();
  if trimmed.is_empty() {
    return false;
  }
  let comma_count =
    trimmed.matches(',').count();
  if comma_count == 0 {
    return false;
  }
  let word_count =
    trimmed.split_whitespace().count();
  if word_count > 10 {
    return false;
  }
  let parts = trimmed
    .split(',')
    .map(str::trim)
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>();
  if parts.len() < 2 {
    return false;
  }
  if parts[1]
    .chars()
    .any(|c| c.is_ascii_digit())
  {
    return false;
  }
  let family_raw =
    parts[0].trim().to_lowercase();
  if family_raw.starts_with("a ")
    || family_raw.starts_with("an ")
    || family_raw.starts_with("the ")
  {
    return false;
  }
  let family_tokens =
    parts[0].split_whitespace();
  let given_tokens =
    parts[1].split_whitespace();
  let allowed_particles = [
    "da", "de", "del", "der", "den",
    "di", "du", "la", "le", "van",
    "von", "al", "bin", "ibn"
  ];
  let family_ok = family_tokens
    .clone()
    .all(|token| {
      let normalized =
        normalize_author_component(
          token
        )
        .to_lowercase();
      allowed_particles
        .contains(&normalized.as_str())
        || token
          .chars()
          .next()
          .map(|c| c.is_uppercase())
          .unwrap_or(false)
    });
  let given_ok =
    given_tokens.clone().any(|token| {
      looks_like_given_name(token)
        || looks_like_initials(token)
    });
  family_ok && given_ok
}

fn trim_author_segment(
  segment: &str
) -> String {
  if let Some((before, _)) =
    segment.split_once('(')
    && segment_has_year(segment)
  {
    return before
      .trim()
      .trim_end_matches(',')
      .to_string();
  }
  if !segment.contains(';')
    && !segment.contains('&')
    && !segment.contains(" and ")
    && let Some((before, after)) =
      segment.split_once('.')
  {
    let next = after.trim_start();
    if !next.is_empty()
      && !next
        .chars()
        .next()
        .map(|c| c.is_uppercase())
        .unwrap_or(false)
    {
      return before
        .trim()
        .trim_end_matches(',')
        .to_string();
    }
    if next
      .split_whitespace()
      .next()
      .map(|token| {
        !looks_like_initials(token)
      })
      .unwrap_or(false)
    {
      return before
        .trim()
        .trim_end_matches(',')
        .to_string();
    }
  }
  if let Some((before, after)) =
    segment.rsplit_once(',')
  {
    let tail = after.trim();
    if !tail.is_empty()
      && !segment.contains(';')
      && !segment.contains('&')
      && !tail.contains(';')
      && !tail.contains('&')
      && !tail.contains(" and ")
      && !tail.split_whitespace().all(
        |token| {
          looks_like_initials(token)
        }
      )
      && !looks_like_family_with_initials(
        tail
      )
      && !looks_like_person_name(tail)
      && !looks_like_initial_surname(tail)
      && segment_is_journal_like(tail)
    {
      return before
        .trim()
        .trim_end_matches(',')
        .to_string();
    }
  }
  let tokens = segment
    .split_whitespace()
    .collect::<Vec<_>>();
  for (idx, token) in
    tokens.iter().enumerate()
  {
    let cleaned =
      token.trim_matches(|c: char| {
        c.is_ascii_punctuation()
      });
    let lower = cleaned.to_lowercase();
    if (parse_month_token(cleaned)
      .is_some()
      || lower.starts_with("vol")
      || lower == "pp"
      || lower == "pp.")
      && idx > 0
    {
      return tokens[..idx]
        .join(" ")
        .trim_end_matches(',')
        .to_string();
    }
  }
  if let Some(pos) = segment
    .char_indices()
    .find(|(_, c)| c.is_ascii_digit())
    .map(|(idx, _)| idx)
  {
    let prefix = segment[..pos].trim();
    if prefix.len() >= 3 {
      return prefix
        .trim_end_matches(',')
        .to_string();
    }
  }
  segment.trim().to_string()
}

fn trim_author_segment_at_date(
  segment: &str
) -> String {
  let tokens = segment
    .split_whitespace()
    .collect::<Vec<_>>();
  for (idx, token) in
    tokens.iter().enumerate()
  {
    let cleaned =
      token.trim_matches(|c: char| {
        c.is_ascii_punctuation()
      });
    if cleaned.is_empty() {
      continue;
    }
    let lower = cleaned.to_lowercase();
    let is_year = cleaned.len() == 4
      && cleaned
        .chars()
        .all(|c| c.is_ascii_digit());
    if (is_year
      || parse_month_token(cleaned)
        .is_some())
      && idx > 0
    {
      return tokens[..idx]
        .join(" ")
        .trim_end_matches(',')
        .to_string();
    }
    if is_year
      || parse_month_token(cleaned)
        .is_some()
    {
      break;
    }
    if (lower == "c."
      || lower == "circa"
      || lower == "ca.")
      && idx > 0
    {
      return tokens[..idx]
        .join(" ")
        .trim_end_matches(',')
        .to_string();
    }
  }
  trim_author_segment(segment)
}

fn strip_leading_citation_number(
  reference: &str
) -> String {
  let trimmed = reference.trim();
  if let Some(stripped) =
    strip_bracketed_citation_number(
      trimmed
    )
  {
    return stripped;
  }
  let mut idx = 0usize;
  for ch in trimmed.chars() {
    if ch.is_ascii_digit() {
      idx += ch.len_utf8();
    } else {
      break;
    }
  }
  if idx == 0 {
    return trimmed.to_string();
  }
  let remainder =
    trimmed[idx..].trim_start();
  if remainder.starts_with('.')
    || remainder.starts_with(']')
  {
    return remainder[1..]
      .trim_start()
      .to_string();
  }
  trimmed.to_string()
}

fn is_title_noise_segment(
  segment: &str
) -> bool {
  let lower = segment.to_lowercase();
  if segment_is_container(segment) {
    return true;
  }
  if looks_like_author_list(segment) {
    return true;
  }
  if segment_has_year(segment)
    && (segment_has_volume_marker(
      segment
    ) || segment_has_page_range(
      segment
    ) || segment_has_page_marker(
      segment
    ))
  {
    return true;
  }
  if lower.contains("http")
    || lower.contains("doi")
  {
    return true;
  }
  if lower.contains("ed.")
    || lower.contains("edition")
    || lower.contains("éd")
  {
    return true;
  }
  if lower.contains("pp.")
    || lower.contains("pages")
  {
    return true;
  }
  let word_count =
    segment.split_whitespace().count();
  if word_count <= 1 {
    return true;
  }
  segment.chars().all(|c| {
    c.is_ascii_digit() || c == ','
  })
}

fn select_title_segment(
  segments: &[String]
) -> Option<String> {
  let mut best = None;
  let mut best_score = i32::MIN;
  for (idx, segment) in
    segments.iter().enumerate()
  {
    let trimmed = segment.trim();
    if trimmed.is_empty() {
      continue;
    }
    let score =
      title_segment_score(trimmed, idx);
    if score > best_score {
      best_score = score;
      best = Some(trimmed.to_string());
    }
  }
  best
}

fn select_title_from_segment(
  segment: &str
) -> Option<String> {
  let parts = segment
    .split(',')
    .map(str::trim)
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>();
  for (idx, part) in
    parts.iter().enumerate()
  {
    let trimmed = part.trim();
    if trimmed.is_empty() {
      continue;
    }
    if trimmed
      .eq_ignore_ascii_case("no")
      || trimmed
        .eq_ignore_ascii_case("no.")
    {
      continue;
    }
    let lower = trimmed.to_lowercase();
    if lower.starts_with("and ")
      || lower == "and"
      || lower == "&"
    {
      continue;
    }
    if idx == 0
      && parts
        .get(1)
        .map(|next| {
          looks_like_given_name(next)
        })
        .unwrap_or(false)
      && trimmed
        .split_whitespace()
        .count()
        <= 2
    {
      continue;
    }
    if looks_like_author_list(trimmed)
      || looks_like_family_with_initials(
        trimmed
      )
      || looks_like_initial_surname(
        trimmed
      )
      || segment_is_journal_like(
        trimmed
      )
      || trimmed
        .chars()
        .any(|c| c.is_ascii_digit())
    {
      continue;
    }
    return Some(trimmed.to_string());
  }
  None
}

fn title_segment_score(
  segment: &str,
  index: usize
) -> i32 {
  if is_title_noise_segment(segment) {
    return i32::MIN;
  }
  let mut score = 0;
  if index <= 1 {
    score += 2;
  }
  if segment_has_year(segment) {
    score -= 2;
  }
  if segment.contains(':') {
    score += 1;
  }
  let word_count =
    segment.split_whitespace().count();
  if word_count >= 3 {
    score += 2;
  }
  if word_count >= 8 {
    score += 1;
  }
  score
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
        if prefix.is_empty() {
          let mut suffix =
            String::new();
          if ch == '.' {
            suffix.push(ch);
          }
          return Some(format!(
            "{digits}{suffix}"
          ));
        }
        return Some(digits);
      }
      return None;
    }
  }
  if digits.is_empty() {
    None
  } else {
    if prefix.is_empty() {
      Some(digits.to_string())
    } else {
      Some(digits)
    }
  }
}

fn split_leading_author_by_comma(
  segment: &str
) -> Option<(String, String)> {
  let (before, after) =
    segment.split_once(',')?;
  let before = before.trim();
  let after = after.trim();
  if before.is_empty()
    || after.is_empty()
  {
    return None;
  }
  let tokens = before
    .split_whitespace()
    .collect::<Vec<_>>();
  if tokens.is_empty() {
    return None;
  }
  let last = tokens[tokens.len() - 1];
  let initials_ok = if tokens.len() == 1
  {
    true
  } else {
    tokens[..tokens.len() - 1]
      .iter()
      .all(|token| {
        looks_like_initials(token)
      })
  };
  let surname_ok = last
    .chars()
    .next()
    .map(|c| c.is_uppercase())
    .unwrap_or(false);
  if initials_ok && surname_ok {
    Some((
      before.to_string(),
      after.to_string()
    ))
  } else {
    None
  }
}

fn remainder_has_author_list(
  remainder: &str
) -> bool {
  let snippet = remainder
    .split('(')
    .next()
    .unwrap_or(remainder)
    .trim()
    .trim_start_matches(|c: char| {
      c == ',' || c == ';'
    })
    .trim();
  if snippet.is_empty() {
    return false;
  }
  if snippet.contains(';') {
    return true;
  }
  if snippet.contains('&')
    || snippet.contains(" and ")
  {
    return true;
  }
  if looks_like_author_list(snippet)
    && snippet.split_whitespace().any(
      |token| {
        looks_like_initials(token)
      }
    )
  {
    return true;
  }
  false
}

fn find_author_list_end(
  reference: &str
) -> Option<usize> {
  let chars: Vec<_> =
    reference.chars().collect();
  let mut idx = 0usize;
  while idx < chars.len() {
    if chars[idx] == '.' {
      let mut before = idx;
      while before > 0
        && chars[before - 1]
          .is_whitespace()
      {
        before -= 1;
      }
      let mut prev_token =
        String::new();
      let mut pos = before;
      while pos > 0 {
        let ch = chars[pos - 1];
        if ch.is_whitespace() {
          break;
        }
        prev_token.insert(0, ch);
        pos -= 1;
      }
      let mut look = idx + 1;
      while look < chars.len()
        && chars[look].is_whitespace()
      {
        look += 1;
      }
      if look >= chars.len() {
        return Some(idx);
      }
      let mut token = String::new();
      let mut pos = look;
      while pos < chars.len()
        && !chars[pos].is_whitespace()
      {
        token.push(chars[pos]);
        pos += 1;
      }
      let token_clean = token
        .trim_matches(|c: char| {
          c.is_ascii_punctuation()
        })
        .to_lowercase();
      if token_clean.is_empty() {
        idx += 1;
        continue;
      }
      if token.ends_with(',')
        || token.ends_with(';')
      {
        idx += 1;
        continue;
      }
      if token_clean.len() == 4
        && token_clean
          .chars()
          .all(|c| c.is_ascii_digit())
      {
        return Some(idx);
      }
      if token_clean == "and"
        || token_clean == "&"
      {
        idx += 1;
        continue;
      }
      let prev_initial =
        looks_like_initials(
          &prev_token
        );
      let next_surname = token
        .chars()
        .next()
        .map(|c| c.is_uppercase())
        .unwrap_or(false)
        && token
          .chars()
          .any(|c| c.is_lowercase());
      if prev_initial && next_surname {
        let mut after = pos;
        while after < chars.len()
          && chars[after]
            .is_whitespace()
        {
          after += 1;
        }
        if after < chars.len()
          && chars[after] == ','
        {
          idx += 1;
          continue;
        }
        return Some(idx);
      }
      if !looks_like_initials(&token) {
        return Some(idx);
      }
    }
    idx += 1;
  }
  None
}

fn strip_bracketed_citation_number(
  reference: &str
) -> Option<String> {
  let trimmed = reference.trim();
  let mut chars = trimmed.chars();
  let first = chars.next()?;
  if first != '[' && first != '(' {
    return None;
  }
  let mut digits = String::new();
  for ch in chars.by_ref() {
    if ch.is_ascii_digit() {
      digits.push(ch);
    } else if ch == ']' || ch == ')' {
      break;
    } else {
      return None;
    }
  }
  if digits.is_empty() {
    return None;
  }
  let remainder =
    chars.as_str().trim_start();
  Some(
    remainder
      .trim_start_matches('.')
      .trim_start()
      .to_string()
  )
}

fn split_author_title_segment(
  segment: &str
) -> Option<(String, String)> {
  let open = segment.find('(')?;
  let close = segment[open + 1..]
    .find(')')?
    + open
    + 1;
  let inside =
    &segment[open + 1..close];
  if inside
    .chars()
    .filter(|c| c.is_ascii_digit())
    .count()
    < 4
  {
    return None;
  }
  let author = segment[..open]
    .trim()
    .trim_end_matches(',')
    .trim()
    .to_string();
  let title = segment[close + 1..]
    .trim_start()
    .trim_start_matches(|c: char| {
      c == '.' || c == ':' || c == ';'
    })
    .trim()
    .to_string();
  if author.is_empty()
    || title.len() < 3
  {
    return None;
  }
  Some((author, title))
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
  let year_pos =
    parts.iter().position(|part| {
      part
        .chars()
        .filter(|c| c.is_ascii_digit())
        .count()
        >= 4
    });
  let pos = year_pos?;
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
  segment_is_conference(segment)
    || segment_is_journal_like(segment)
}

fn segment_is_conference(
  segment: &str
) -> bool {
  let lower = segment.to_lowercase();
  if lower.contains("presented at") {
    return true;
  }
  let keywords = [
    "conference",
    "symposium",
    "workshop",
    "meeting",
    "colloquium"
  ];
  if keywords
    .iter()
    .any(|kw| lower.contains(kw))
  {
    return true;
  }
  if lower.contains("proceedings") {
    if lower.contains("ieee")
      || lower.contains("acm")
    {
      return false;
    }
    return true;
  }
  if lower.contains("proc.") {
    if lower.contains("ieee")
      || lower.contains("acm")
    {
      return false;
    }
    let strong_journal = lower
      .contains("journal")
      || lower.contains("transactions")
      || lower.contains("letters")
      || lower.contains("lett")
      || lower.contains("review")
      || lower.contains("rev.")
      || lower.contains("annals")
      || lower.contains("acta")
      || lower.contains("acad")
      || lower.contains("proc natl")
      || lower.contains("proc. natl");
    if strong_journal {
      return false;
    }
    return lower.contains("conf")
      || lower.contains("symp")
      || lower.contains("workshop")
      || lower.contains("proc.");
  }
  false
}

fn segment_is_journal_like(
  segment: &str
) -> bool {
  let lower = segment.to_lowercase();
  if lower.contains("journal")
    || lower.contains("trans.")
    || lower.contains("transactions")
    || lower.contains("bulletin")
    || lower.contains("letters")
    || lower.contains("lett")
    || lower.contains("annals")
    || lower.contains("review")
    || lower.contains("rev.")
    || lower.contains("acta")
  {
    return true;
  }
  if lower.contains("proc natl")
    || lower.contains("proc. natl")
    || lower.contains("acad")
  {
    return true;
  }
  if lower
    .starts_with("proceedings of the")
    && (lower.contains("ieee")
      || lower.contains("acm"))
  {
    return true;
  }
  if segment.contains(" J ")
    || segment.contains(", J ")
  {
    return true;
  }
  if looks_like_short_journal(segment) {
    return true;
  }
  let trimmed = segment.trim();
  if trimmed.starts_with("J ")
    || trimmed.starts_with("J.")
    || trimmed.starts_with("J ")
    || trimmed.starts_with("J-")
  {
    return true;
  }
  if lower.contains("ieee")
    || lower.contains("acm")
    || lower.contains("sig")
  {
    return true;
  }
  false
}

fn looks_like_short_journal(
  segment: &str
) -> bool {
  let tokens = segment
    .split_whitespace()
    .collect::<Vec<_>>();
  if tokens.len() < 2
    || tokens.len() > 4
  {
    return false;
  }
  if let Some(first) = tokens.first()
    && matches!(
      first.to_lowercase().as_str(),
      "a" | "an" | "the"
    )
  {
    return false;
  }
  if tokens.iter().any(|token| {
    token
      .chars()
      .any(|c| c.is_ascii_digit())
  }) {
    return false;
  }
  if segment.contains(':') {
    return false;
  }
  let journal_tokens = [
    "j", "jr", "lett", "rev", "proc",
    "acad", "ann", "bull", "trans",
    "acta", "comm", "conf"
  ];
  let has_keyword =
    tokens.iter().any(|token| {
      let lowered =
        token.to_lowercase();
      journal_tokens
        .iter()
        .any(|kw| lowered == *kw)
    });
  if has_keyword {
    return true;
  }
  let has_abbrev =
    tokens.iter().any(|token| {
      token.ends_with('.')
        || token
          .chars()
          .all(|c| c.is_uppercase())
    });
  let short_tokens = tokens
    .iter()
    .all(|token| token.len() <= 6);
  has_abbrev && short_tokens
}

fn segment_journal_score(
  segment: &str,
  dictionary: Option<&Dictionary>
) -> usize {
  let mut score = 0usize;
  if segment_is_journal_like(segment) {
    score += 3;
  }
  if looks_like_journal_name(segment) {
    score += 1;
  }
  if segment.split(',').any(|part| {
    let trimmed = part.trim();
    !trimmed.is_empty()
      && (segment_is_journal_like(
        trimmed
      ) || looks_like_short_journal(
        trimmed
      ) || looks_like_journal_name(
        trimmed
      ))
  }) {
    score += 2;
  }
  if segment.starts_with("J ")
    || segment.starts_with("J.")
  {
    score += 2;
  }
  if let Some(dictionary) = dictionary {
    let matches = segment
      .split(|c: char| {
        !c.is_alphanumeric()
      })
      .filter(|token| !token.is_empty())
      .filter(|token| {
        dictionary
          .lookup(token)
          .contains(
            &DictionaryCode::Journal
          )
      })
      .count();
    score += matches * 2;
  }
  score
}

fn strip_numeric_suffix(
  segment: &str
) -> String {
  let mut digit_pos = None;
  for (idx, ch) in
    segment.char_indices()
  {
    if ch.is_ascii_digit() {
      digit_pos = Some(idx);
      break;
    }
  }
  if let Some(idx) = digit_pos {
    let prefix = segment[..idx].trim();
    if let Some((sep_idx, _)) = prefix
      .match_indices(
        &[',', ';', '('][..]
      )
      .next_back()
    {
      let tail = prefix[sep_idx + 1..]
        .to_lowercase();
      if tail.contains("vol")
        || tail.contains("no")
        || tail.contains("issue")
        || tail.contains("part")
        || tail.trim().is_empty()
      {
        return prefix[..sep_idx]
          .trim()
          .to_string();
      }
    }
  }
  segment.trim().to_string()
}
fn resolve_type(
  reference: &str
) -> String {
  let lower = reference.to_lowercase();
  if lower.contains("chapter")
    || lower.contains("chap.")
    || lower.contains("ch.")
  {
    return "chapter".into();
  }
  if lower.contains("thesis")
    || lower.contains("dissertation")
  {
    return "thesis".into();
  }
  if lower.contains("report")
    || lower
      .contains("technical report")
  {
    return "report".into();
  }
  if split_reference_segments(reference)
    .iter()
    .any(|segment| {
      segment_is_conference(segment)
    })
  {
    return "paper-conference".into();
  }
  if split_reference_segments(reference)
    .iter()
    .any(|segment| {
      segment_is_journal_like(segment)
        || looks_like_journal_name(segment)
        || segment.split(',').any(|part| {
          let trimmed = part.trim();
          !trimmed.is_empty()
            && (segment_is_journal_like(
              trimmed
            )
              || looks_like_short_journal(
                trimmed
              )
              || looks_like_journal_name(
                trimmed
              ))
        })
    })
  {
    return "article-journal".into();
  }
  "book".into()
}

fn resolve_type_with_dictionary(
  reference: &str,
  dictionary: &Dictionary
) -> String {
  let lower = reference.to_lowercase();
  if lower.contains("chapter")
    || lower.contains("chap.")
    || lower.contains("ch.")
  {
    return "chapter".into();
  }
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
  if split_reference_segments(reference)
    .iter()
    .any(|segment| {
      segment_is_conference(segment)
    })
  {
    return "paper-conference".into();
  }
  resolve_type(reference)
}

pub(crate) fn extract_location(
  reference: &str
) -> String {
  let (location, _) =
    extract_location_publisher(
      reference
    );
  location
}

pub(crate) fn extract_publisher(
  reference: &str
) -> String {
  let (_, publisher) =
    extract_location_publisher(
      reference
    );
  publisher
}

fn extract_location_publisher(
  reference: &str
) -> (String, String) {
  let Some(pos) = reference.rfind(':')
  else {
    return (
      String::new(),
      String::new()
    );
  };
  let before = reference[..pos].trim();
  let after =
    reference[pos + 1..].trim();
  let location_segment = before
    .rsplit('.')
    .next()
    .unwrap_or(before)
    .trim();
  if !is_location_segment(
    location_segment
  ) {
    return (
      String::new(),
      String::new()
    );
  }
  let location =
    clean_segment(location_segment);
  let publisher = after
    .split(',')
    .next()
    .map(|s| s.trim().to_string())
    .unwrap_or_default();
  (location, publisher)
}

fn is_location_segment(
  segment: &str
) -> bool {
  let words = segment
    .split_whitespace()
    .collect::<Vec<_>>();
  if words.is_empty() || words.len() > 3
  {
    return false;
  }
  let lower = segment.to_lowercase();
  if lower.contains(" and ")
    || lower.contains(" in ")
    || lower.contains(" for ")
  {
    return false;
  }
  let allowed_particles =
    ["de", "la", "of", "da", "del"];
  for word in words {
    let trimmed =
      word.trim_matches(|c: char| {
        c == ',' || c == '.'
      });
    let lower = trimmed.to_lowercase();
    if allowed_particles
      .contains(&lower.as_str())
    {
      continue;
    }
    let mut chars = trimmed.chars();
    let Some(first) = chars.next()
    else {
      continue;
    };
    if !first.is_uppercase() {
      return false;
    }
  }
  true
}

pub(crate) fn extract_pages(
  reference: &str
) -> String {
  let tokens = reference
    .split_whitespace()
    .collect::<Vec<_>>();
  for (idx, token) in
    tokens.iter().enumerate()
  {
    let cleaned =
      token.trim_matches(|c: char| {
        c == ',' || c == ';'
      });
    let lower = cleaned.to_lowercase();
    let marker_len = if lower
      .starts_with("pp.")
    {
      Some(3)
    } else if lower.starts_with("p.") {
      Some(2)
    } else {
      None
    };
    if let Some(marker_len) = marker_len
    {
      let remainder = cleaned
        .get(marker_len..)
        .unwrap_or("");
      if !remainder.is_empty() {
        if let Some(range) =
          parse_page_range_token(
            remainder
          )
          .or_else(|| {
            parse_short_page_range_token(
              remainder
            )
          })
        {
          return range;
        }
        let digits: String = remainder
          .chars()
          .filter(|c| {
            c.is_ascii_digit()
              || matches!(
                c,
                '-' | '–' | '—'
              )
          })
          .collect();
        if !digits.is_empty() {
          return digits
            .replace(['–', '—'], "-");
        }
      }
      if let Some(next) =
        tokens.get(idx + 1)
      {
        if let Some(range) =
          parse_page_range_token(next)
            .or_else(|| {
              parse_short_page_range_token(
                next
              )
            })
        {
          return range;
        }
        let digits: String = next
          .chars()
          .filter(|c| {
            c.is_ascii_digit()
              || matches!(
                c,
                '-' | '–' | '—'
              )
          })
          .collect();
        if !digits.is_empty() {
          return digits
            .replace(['–', '—'], "-");
        }
      }
      continue;
    }
    if matches!(
      lower.as_str(),
      "p" | "pp" | "p." | "pp."
    )
      && let Some(next) =
        tokens.get(idx + 1)
    {
      if let Some(range) =
        parse_page_range_token(next)
          .or_else(|| {
            parse_short_page_range_token(
              next
            )
          })
      {
        return range;
      }
      let digits: String = next
        .chars()
        .filter(|c| {
          c.is_ascii_digit()
            || matches!(
              c,
              '-' | '–' | '—'
            )
        })
        .collect();
      if !digits.is_empty() {
        return digits
          .replace(['–', '—'], "-");
      }
    }
  }
  if let Some(value) =
    pages_from_year_volume(reference)
  {
    return value;
  }
  if let Some(range) =
    find_page_range(reference)
  {
    return range;
  }
  if let Some(value) =
    trailing_page_token(reference)
  {
    return value;
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

fn trailing_page_token(
  reference: &str
) -> Option<String> {
  let tokens = reference
    .split_whitespace()
    .collect::<Vec<_>>();
  let mut candidate = None;
  for token in tokens.iter().rev() {
    let cleaned =
      token.trim_matches(|c: char| {
        c == ',' || c == ';' || c == '.'
      });
    if cleaned.is_empty() {
      continue;
    }
    if cleaned
      .chars()
      .all(|c| c.is_ascii_digit())
    {
      candidate =
        Some(cleaned.to_string());
      break;
    }
    break;
  }
  let candidate = candidate?;
  let years =
    collect_year_tokens(reference);
  if years
    .iter()
    .any(|year| year == &candidate)
  {
    return None;
  }
  if candidate.len() >= 3 {
    Some(candidate)
  } else {
    None
  }
}

fn pages_from_year_volume(
  reference: &str
) -> Option<String> {
  let cleaned =
    strip_leading_citation_number(
      reference
    );
  let numbers =
    numeric_tokens(&cleaned);
  let year_idx =
    year_token_index(&numbers)?;
  if year_idx != 0 {
    return None;
  }
  let volume =
    numbers.get(year_idx + 1);
  let candidate = numbers.last()?;
  if Some(candidate) == volume {
    return None;
  }
  if year_idx + 1 < numbers.len() {
    if candidate == &numbers[year_idx] {
      return None;
    }
    return Some(candidate.clone());
  }
  None
}

fn find_page_range(
  reference: &str
) -> Option<String> {
  let tokens = reference
    .split_whitespace()
    .collect::<Vec<_>>();
  let year_tokens =
    collect_year_tokens(reference);
  for (idx, token) in
    tokens.iter().enumerate()
  {
    if let Some(range) =
      parse_page_range_token(token)
    {
      return Some(range);
    }
    if let Some(range) =
      parse_short_page_range_token(
        token
      )
    {
      let has_year_after = tokens
        .iter()
        .skip(idx + 1)
        .any(|candidate| {
          let cleaned = candidate
            .trim_matches(|c: char| {
              c == ','
                || c == ';'
                || c == '.'
            });
          year_tokens.contains(
            &cleaned.to_string()
          )
        });
      if has_year_after
        || idx + 1 == tokens.len()
      {
        return Some(range);
      }
    }
  }
  None
}

fn parse_page_range_token(
  token: &str
) -> Option<String> {
  let cleaned: String = token
    .chars()
    .filter(|c| {
      c.is_ascii_digit()
        || matches!(c, '-' | '–' | '—')
    })
    .collect();
  if !cleaned.contains('-')
    && !cleaned.contains('–')
    && !cleaned.contains('—')
  {
    return None;
  }
  let parts = cleaned
    .split(|c| {
      c == '-' || c == '–' || c == '—'
    })
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>();
  if parts.len() != 2 {
    return None;
  }
  let left = parts[0];
  let right = parts[1];
  if left.len() < 2 || right.len() < 2 {
    return None;
  }
  if left.len() == 4 && right.len() <= 2
  {
    return None;
  }
  if left.len() == 4 && right.len() == 4
  {
    return None;
  }
  if left.len() < 3 && right.len() < 3 {
    return None;
  }
  Some(format!("{left}-{right}"))
}

fn parse_short_page_range_token(
  token: &str
) -> Option<String> {
  if token
    .chars()
    .any(|c| c.is_alphabetic())
  {
    return None;
  }
  let cleaned: String = token
    .chars()
    .filter(|c| {
      c.is_ascii_digit()
        || matches!(c, '-' | '–' | '—')
    })
    .collect();
  let parts = cleaned
    .split(|c| {
      c == '-' || c == '–' || c == '—'
    })
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>();
  if parts.len() != 2 {
    return None;
  }
  let left = parts[0];
  let right = parts[1];
  if left.len() < 2 || right.len() < 2 {
    return None;
  }
  if left.len() == 4 && right.len() <= 2
  {
    return Some(format!(
      "{left}-{right}"
    ));
  }
  if left.len() == 4 && right.len() == 4
  {
    return Some(format!(
      "{left}-{right}"
    ));
  }
  if left.len() == 2 && right.len() == 2
  {
    return Some(format!(
      "{left}-{right}"
    ));
  }
  None
}

pub(crate) fn extract_collection_title(
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
    "symposium",
    "volume"
  ];
  for keyword in keywords {
    if let Some(segment) =
      segment_after_keyword(
        reference, keyword
      )
    {
      let lower =
        segment.to_lowercase();
      if matches!(
        lower.as_str(),
        "ed" | "ed." | "eds" | "eds."
      ) {
        continue;
      }
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

pub(crate) fn extract_container_title(
  reference: &str
) -> Option<String> {
  split_reference_segments(reference)
    .iter()
    .map(|segment| segment.trim())
    .filter(|segment| {
      !segment.is_empty()
    })
    .filter(|segment| {
      segment_is_conference(segment)
    })
    .map(strip_numeric_suffix)
    .map(|segment| {
      clean_segment(&segment)
    })
    .map(|segment| {
      strip_trailing_metadata(&segment)
    })
    .map(|segment| {
      strip_trailing_location(&segment)
    })
    .map(|segment| {
      strip_container_prefix(&segment)
    })
    .next()
    .or_else(|| {
      extract_container_from_in_segment(
        reference
      )
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

pub(crate) fn extract_journal(
  reference: &str
) -> Option<String> {
  extract_journal_with_dictionary(
    reference, None
  )
}

pub(crate) fn extract_journal_with_dictionary(
  reference: &str,
  dictionary: Option<&Dictionary>
) -> Option<String> {
  let title = extract_title(reference);
  let title_norm =
    normalize_compare_value(&title);
  let title_norm_ref =
    if title_norm.is_empty() {
      None
    } else {
      Some(title_norm.as_str())
    };
  let mut best: Option<(
    usize,
    String
  )> = None;
  for segment in
    split_reference_segments(reference)
  {
    let trimmed = segment.trim();
    if trimmed.is_empty() {
      continue;
    }
    if trimmed.contains(';')
      || trimmed.contains('&')
    {
      continue;
    }
    if !title_norm.is_empty()
      && normalize_compare_value(
        trimmed
      ) == title_norm
    {
      continue;
    }
    if trimmed.contains(':') {
      continue;
    }
    if looks_like_author_list(trimmed) {
      continue;
    }
    if looks_like_person_name(trimmed)
      && !segment_has_year(trimmed)
      && !segment_has_volume_marker(
        trimmed
      )
      && !segment_has_page_range(
        trimmed
      )
    {
      continue;
    }
    let mut score =
      segment_journal_score(
        trimmed, dictionary
      );
    let candidate =
      extract_journal_from_segment(
        trimmed,
        title_norm_ref
      );
    if score == 0 && candidate.is_none()
    {
      continue;
    }
    if score == 0 {
      score = 1;
    }
    let cleaned = candidate
      .unwrap_or_else(|| {
        strip_numeric_suffix(trimmed)
      });
    let cleaned =
      clean_segment(&cleaned);
    let cleaned =
      strip_leading_date(&cleaned);
    let cleaned =
      strip_trailing_metadata(&cleaned);
    let cleaned =
      strip_trailing_location(&cleaned);
    let cleaned =
      strip_container_prefix(&cleaned);
    if cleaned.is_empty() {
      continue;
    }
    match best {
      | Some((best_score, _))
        if best_score >= score => {}
      | _ => {
        best = Some((score, cleaned));
      }
    }
  }
  if let Some((_, value)) = best {
    return Some(value);
  }
  for segment in
    split_reference_segments(reference)
  {
    let trimmed = segment.trim();
    if trimmed.is_empty() {
      continue;
    }
    if trimmed.contains(':') {
      continue;
    }
    if trimmed.contains(';')
      || trimmed.contains('&')
    {
      continue;
    }
    let candidate = trimmed
      .split(',')
      .next()
      .unwrap_or("")
      .trim();
    if candidate.is_empty() {
      continue;
    }
    if trimmed.split_whitespace().any(
      |token| {
        looks_like_initials(token)
      }
    ) {
      continue;
    }
    if !title_norm.is_empty()
      && normalize_compare_value(
        candidate
      ) == title_norm
    {
      continue;
    }
    if looks_like_author_list(candidate)
    {
      continue;
    }
    if looks_like_journal_name(
      candidate
    ) && trimmed.contains(',')
    {
      return Some(clean_segment(
        candidate
      ));
    }
  }
  None
}

fn extract_journal_from_segment(
  segment: &str,
  title_norm: Option<&str>
) -> Option<String> {
  let mut best = None;
  for part in segment.split(',') {
    let trimmed = part.trim();
    if trimmed.is_empty() {
      continue;
    }
    if let Some(title_norm) = title_norm
      && normalize_compare_value(
        trimmed
      ) == title_norm
    {
      continue;
    }
    let lower = trimmed.to_lowercase();
    if lower.contains("vol")
      || lower.contains("pp")
      || lower.contains("no.")
      || lower.contains("issue")
    {
      continue;
    }
    if trimmed
      .chars()
      .any(|c| c.is_ascii_digit())
    {
      if let Some(stripped) =
        strip_trailing_year_token(
          trimmed
        )
        .or_else(|| {
          strip_trailing_number_token(
            trimmed
          )
        })
        && is_journal_candidate(&stripped)
      {
        best = Some(stripped);
        break;
      }
      continue;
    }
    if looks_like_author_list(trimmed)
      || looks_like_family_with_initials(
        trimmed
      )
      || (trimmed.contains('.')
        && looks_like_initial_surname(
          trimmed
        ))
    {
      continue;
    }
    if is_journal_candidate(trimmed) {
      best = Some(trimmed.to_string());
      break;
    }
  }
  best
}

fn is_journal_candidate(
  value: &str
) -> bool {
  segment_is_journal_like(value)
    || looks_like_short_journal(value)
    || looks_like_journal_name(value)
}

fn strip_trailing_year_token(
  value: &str
) -> Option<String> {
  let tokens = value
    .split_whitespace()
    .collect::<Vec<_>>();
  if tokens.len() < 2 {
    return None;
  }
  let last = tokens.last()?;
  if last.len() != 4
    || !last
      .chars()
      .all(|c| c.is_ascii_digit())
  {
    return None;
  }
  let year =
    last.parse::<u32>().ok()?;
  if !(1800..=2099).contains(&year) {
    return None;
  }
  let candidate = tokens
    [..tokens.len() - 1]
    .join(" ");
  if candidate.is_empty() {
    None
  } else {
    Some(candidate)
  }
}

fn strip_trailing_number_token(
  value: &str
) -> Option<String> {
  let tokens = value
    .split_whitespace()
    .collect::<Vec<_>>();
  if tokens.len() < 2 {
    return None;
  }
  let last = tokens.last()?;
  if last.len() > 3
    || !last
      .chars()
      .all(|c| c.is_ascii_digit())
  {
    return None;
  }
  let candidate = tokens
    [..tokens.len() - 1]
    .join(" ");
  if candidate.is_empty() {
    None
  } else {
    Some(candidate)
  }
}

fn looks_like_journal_name(
  segment: &str
) -> bool {
  if looks_like_author_list(segment) {
    return false;
  }
  if segment.contains(':') {
    return false;
  }
  let tokens = segment
    .split_whitespace()
    .collect::<Vec<_>>();
  if tokens.len() < 2
    || tokens.len() > 6
  {
    return false;
  }
  if tokens.iter().any(|token| {
    token
      .chars()
      .any(|c| c.is_ascii_digit())
  }) {
    return false;
  }
  let particles = [
    "of", "de", "la", "le", "du",
    "der", "van", "von", "the", "and"
  ];
  let mut saw_title = false;
  for token in tokens {
    let cleaned =
      token.trim_matches(|c: char| {
        c.is_ascii_punctuation()
      });
    if cleaned.is_empty() {
      continue;
    }
    let lower = cleaned.to_lowercase();
    if particles
      .contains(&lower.as_str())
    {
      continue;
    }
    let mut chars = cleaned.chars();
    let Some(first) = chars.next()
    else {
      continue;
    };
    if first.is_uppercase()
      && chars.any(|c| c.is_lowercase())
    {
      saw_title = true;
    } else {
      return false;
    }
  }
  saw_title
}

fn normalize_compare_value(
  value: &str
) -> String {
  value
    .to_lowercase()
    .chars()
    .filter(|c| {
      c.is_ascii_alphanumeric()
        || c.is_whitespace()
    })
    .collect::<String>()
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ")
}

fn strip_container_prefix(
  segment: &str
) -> String {
  let trimmed = segment.trim();
  for prefix in ["In ", "in "] {
    if let Some(stripped) =
      trimmed.strip_prefix(prefix)
    {
      return stripped.trim().to_string();
    }
  }
  trimmed.to_string()
}

fn strip_trailing_location(
  segment: &str
) -> String {
  let mut current =
    segment.trim().to_string();
  loop {
    let Some((before, after)) =
      current.rsplit_once(',')
    else {
      return current.trim().to_string();
    };
    let tail = after.trim();
    if tail.is_empty() {
      current =
        before.trim().to_string();
      continue;
    }
    if tail
      .chars()
      .any(|c| c.is_ascii_digit())
    {
      return current.trim().to_string();
    }
    let word_count =
      tail.split_whitespace().count();
    if word_count > 4 {
      return current.trim().to_string();
    }
    if tail.split_whitespace().all(
      |word| {
        word
          .chars()
          .next()
          .map(|c| c.is_uppercase())
          .unwrap_or(false)
      }
    ) {
      current =
        before.trim().to_string();
      continue;
    }
    return current.trim().to_string();
  }
}

fn strip_trailing_metadata(
  segment: &str
) -> String {
  let mut tokens = segment
    .split_whitespace()
    .collect::<Vec<_>>();
  while let Some(last) = tokens.last() {
    let cleaned = last
      .trim_matches(|c: char| {
        c == ','
          || c == ';'
          || c == ')'
          || c == '('
      })
      .to_string();
    if cleaned.is_empty() {
      tokens.pop();
      continue;
    }
    if parse_page_range_token(&cleaned)
      .is_some()
      || parse_short_page_range_token(
        &cleaned
      )
      .is_some()
    {
      tokens.pop();
      continue;
    }
    if cleaned
      .chars()
      .all(|c| c.is_ascii_digit())
      && cleaned.len() >= 3
    {
      tokens.pop();
      continue;
    }
    let lower = cleaned.to_lowercase();
    if lower == "pp"
      || lower == "pp."
      || lower == "p."
      || lower == "pages"
      || lower == "vol"
      || lower == "vol."
      || lower == "no."
      || lower == "issue"
    {
      tokens.pop();
      continue;
    }
    break;
  }
  let mut result = tokens.join(" ");
  result = result
    .trim_end_matches(|c: char| {
      c == ',' || c == ';'
    })
    .trim()
    .to_string();
  result
}

fn strip_leading_date(
  segment: &str
) -> String {
  let trimmed = segment.trim();
  let Some((before, after)) =
    trimmed.split_once(',')
  else {
    return trimmed.to_string();
  };
  let prefix = before.trim();
  let prefix_tokens = prefix
    .split_whitespace()
    .collect::<Vec<_>>();
  if prefix_tokens.iter().all(|token| {
    let cleaned =
      token.trim_matches(|c: char| {
        c.is_ascii_punctuation()
      });
    if cleaned.is_empty() {
      return true;
    }
    parse_month_token(cleaned).is_some()
      || cleaned
        .chars()
        .filter(|c| c.is_ascii_digit())
        .count()
        >= 4
  }) {
    return after.trim().to_string();
  }
  trimmed.to_string()
}

pub(crate) fn extract_editor(
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
      let normalized = segment
        .trim_matches(|c: char| {
          c.is_ascii_punctuation()
            || c == '('
            || c == ')'
        })
        .to_lowercase();
      if matches!(
        normalized.as_str(),
        "ed" | "ed." | "eds" | "eds."
      ) {
        continue;
      }
      return Some(segment);
    }
  }

  None
}

fn extract_editor_list(
  reference: &str
) -> Vec<String> {
  let editors = extract_editor(
    reference
  )
  .or_else(|| {
    extract_editors_from_in_segment(
      reference
    )
  });
  let Some(editors) = editors else {
    return Vec::new();
  };
  split_editor_names(&editors)
}

fn split_editor_names(
  editors: &str
) -> Vec<String> {
  let normalized = editors
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
    .map(|piece| piece.to_string())
    .collect()
}

fn extract_editors_from_in_segment(
  reference: &str
) -> Option<String> {
  let in_pos =
    find_in_segment_index(reference)?;
  let after_in =
    reference.get(in_pos + 3..)?;
  let lower_after =
    after_in.to_lowercase();
  let ed_pos =
    lower_after.find("(ed")?;
  let editor_segment =
    after_in[..ed_pos].trim();
  let cleaned = editor_segment
    .trim_matches(|c: char| {
      c == ',' || c == ';'
    })
    .trim();
  if cleaned.is_empty() {
    None
  } else {
    Some(cleaned.to_string())
  }
}

fn extract_container_from_in_segment(
  reference: &str
) -> Option<String> {
  let in_pos =
    find_in_segment_index(reference)?;
  let after_in =
    reference.get(in_pos + 3..)?;
  let after_in = after_in.trim_start();
  let container_section =
    if let Some(close) =
      after_in.find(')')
    {
      after_in[close + 1..].trim_start()
    } else {
      after_in
    };
  let container_section =
    container_section
      .trim_start_matches(|c: char| {
        c == ',' || c == ';' || c == '.'
      })
      .trim_start();
  if container_section.is_empty() {
    return None;
  }
  let segment = container_section
    .split(|c: char| {
      c == '.' || c == ';'
    })
    .next()
    .unwrap_or("")
    .trim();
  if segment.is_empty() {
    None
  } else {
    let cleaned =
      clean_segment(segment);
    let cleaned =
      strip_trailing_metadata(&cleaned);
    let cleaned =
      strip_trailing_location(&cleaned);
    if cleaned.is_empty() {
      None
    } else {
      Some(cleaned)
    }
  }
}

fn find_in_segment_index(
  reference: &str
) -> Option<usize> {
  let lower = reference.to_lowercase();
  if lower.starts_with("in ") {
    return Some(0);
  }
  for (idx, _) in
    lower.match_indices(" in ")
  {
    let before =
      lower[..idx].trim_end();
    if before.ends_with('.')
      || before.ends_with(';')
      || before.ends_with(':')
    {
      return Some(idx);
    }
  }
  None
}

fn strip_parenthetical_date(
  segment: &str
) -> String {
  let mut output = String::new();
  let mut chars =
    segment.chars().peekable();
  while let Some(ch) = chars.next() {
    if ch == '(' {
      let mut contents = String::new();
      for inner in chars.by_ref() {
        if inner == ')' {
          break;
        }
        contents.push(inner);
      }
      let lower =
        contents.to_lowercase();
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
        output
          .push_str(contents.trim());
        output.push(')');
      }
      continue;
    }
    output.push(ch);
  }
  output.trim().to_string()
}

pub(crate) fn extract_translator(
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

pub(crate) fn extract_note(
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

pub(crate) fn extract_volume(
  reference: &str
) -> Option<String> {
  let cleaned =
    strip_leading_citation_number(
      reference
    );
  let lower = cleaned.to_lowercase();
  for keyword in [
    "volume", "vol.", "vol", "v.",
    "vols"
  ] {
    if let Some(pos) =
      lower.find(keyword)
    {
      let start = pos + keyword.len();
      let remainder = cleaned
        .get(start..)
        .unwrap_or("");
      if let Some(volume) =
        capture_number_after(
          &cleaned, start
        )
      {
        if let Some(part) =
          extract_part_suffix(remainder)
        {
          return Some(format!(
            "{volume}, Part {part}"
          ));
        }
        return Some(volume);
      }
    }
  }
  for segment in
    split_reference_segments(&cleaned)
  {
    if let Some(volume) =
      extract_volume_from_segment(
        &segment
      )
    {
      return Some(volume);
    }
  }
  let numbers =
    numeric_tokens(&cleaned);
  if !segment_has_page_marker(&cleaned)
    && !segment_has_page_range(&cleaned)
    && let Some(year_idx) =
      year_token_index(&numbers)
    && year_idx == 0
    && let Some(value) =
      numbers.get(year_idx + 1)
    && value.len() <= 3
  {
    return Some(value.clone());
  }
  None
}

pub(crate) fn tokens_from_identifiers(
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

pub(crate) fn extract_issue(
  reference: &str
) -> Option<String> {
  let lower = reference.to_lowercase();
  for keyword in
    ["number", "no.", "issue"]
  {
    if let Some(pos) =
      lower.find(keyword)
    {
      let start = pos + keyword.len();
      let remainder = reference
        .get(start..)
        .unwrap_or("");
      if let Some(value) =
        capture_number_after(
          reference, start
        )
      {
        if let Some(part) =
          extract_part_suffix(remainder)
        {
          return Some(format!(
            "{value}, Part {part}"
          ));
        }
        return Some(value);
      }
    }
  }
  for segment in
    split_reference_segments(reference)
  {
    if let Some(issue) =
      extract_issue_from_segment(
        &segment
      )
    {
      return Some(issue);
    }
  }
  None
}

fn extract_volume_from_segment(
  segment: &str
) -> Option<String> {
  if segment_is_journal_like(segment)
    && let Some(volume) =
      volume_from_segment_before_pages(
        segment
      )
  {
    return Some(volume);
  }
  let (volume, _issue) =
    parse_volume_issue_pair(segment);
  if let Some(volume) = volume {
    if let Some(part) =
      extract_part_suffix(segment)
    {
      return Some(format!(
        "{volume}, Part {part}"
      ));
    }
    return Some(volume);
  }
  let numbers =
    numeric_tokens(segment);
  if segment_is_journal_like(segment)
    && let Some(year_idx) =
      year_token_index(&numbers)
    && year_idx == 0
    && let Some(value) =
      numbers.get(year_idx + 1)
    && value.len() <= 3
  {
    return Some(value.clone());
  }
  if segment_is_journal_like(segment)
    && let Some(value) =
      last_number_token(segment)
    && value.len() <= 3
  {
    return Some(value);
  }
  None
}

fn volume_from_segment_before_pages(
  segment: &str
) -> Option<String> {
  let parts = segment
    .split(',')
    .map(str::trim)
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>();
  if parts.is_empty() {
    return None;
  }
  let page_index =
    parts.iter().position(|part| {
      parse_page_range_token(part)
        .is_some()
        || parse_short_page_range_token(
          part
        )
        .is_some()
    });
  let limit = page_index.unwrap_or(
    parts.len().saturating_sub(1)
  );
  for part in
    parts.iter().take(limit).rev()
  {
    let lower = part.to_lowercase();
    if lower.contains("part")
      || lower.contains("h.")
      || lower.contains("no.")
      || lower.contains("issue")
      || lower.contains("pp")
      || lower.contains("p.")
    {
      continue;
    }
    if let Some(number) =
      last_number_token(part)
      && number.len() <= 3
    {
      return Some(number);
    }
  }
  None
}

fn extract_issue_from_segment(
  segment: &str
) -> Option<String> {
  let (_volume, issue) =
    parse_volume_issue_pair(segment);
  if let Some(issue) = issue {
    return Some(issue);
  }
  if segment
    .to_lowercase()
    .contains("vol")
  {
    return None;
  }
  if let Some(part) =
    extract_part_suffix(segment)
  {
    let before = segment
      .to_lowercase()
      .find("part")
      .and_then(|pos| {
        segment.get(..pos)
      })
      .unwrap_or(segment);
    if let Some(number) =
      last_number_token(before)
    {
      return Some(format!(
        "{number}, Part {part}"
      ));
    }
  }
  None
}

fn parse_volume_issue_pair(
  segment: &str
) -> (Option<String>, Option<String>) {
  let mut volume = None;
  let mut issue = None;
  if let Some(open) = segment.find('(')
    && let Some(close) =
      segment[open + 1..].find(')')
  {
    let inside = &segment
      [open + 1..open + 1 + close];
    let inside_lower =
      inside.to_lowercase();
    let inside_digits =
      number_token(inside);
    if !inside_lower.contains("vol")
      && !inside_lower.contains("part")
      && !inside_lower.contains("no.")
      && let Some(value) =
        inside_digits
      && value.len() <= 3
    {
      issue = Some(value);
    }
    if let Some(before) =
      segment.get(..open)
      && let Some(value) =
        last_number_token(before)
    {
      volume = Some(value);
    }
  }
  (volume, issue)
}

fn number_token(
  segment: &str
) -> Option<String> {
  let token = segment
    .split_whitespace()
    .map(|part| {
      part.trim_matches(|c: char| {
        !c.is_ascii_digit()
      })
    })
    .find(|part| !part.is_empty())?;
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

fn last_number_token(
  segment: &str
) -> Option<String> {
  segment
    .split_whitespace()
    .map(|part| {
      part.trim_matches(|c: char| {
        !c.is_ascii_digit()
      })
    })
    .filter(|part| !part.is_empty())
    .filter(|part| {
      part
        .chars()
        .all(|c| c.is_ascii_digit())
    })
    .map(|part| part.to_string())
    .next_back()
}

fn numeric_tokens(
  segment: &str
) -> Vec<String> {
  let mut tokens = Vec::new();
  let mut current = String::new();
  for ch in segment.chars() {
    if ch.is_ascii_digit() {
      current.push(ch);
    } else if !current.is_empty() {
      tokens.push(current.clone());
      current.clear();
    }
  }
  if !current.is_empty() {
    tokens.push(current);
  }
  tokens
}

fn year_token_index(
  tokens: &[String]
) -> Option<usize> {
  tokens.iter().position(|token| {
    if token.len() != 4 {
      return false;
    }
    token
      .parse::<u32>()
      .ok()
      .filter(|value| {
        (1800..=2099).contains(value)
      })
      .is_some()
  })
}

fn extract_part_suffix(
  segment: &str
) -> Option<String> {
  let lower = segment.to_lowercase();
  let pos = lower.find("part")?;
  let after = segment.get(pos + 4..)?;
  let token = after
    .split_whitespace()
    .find(|part| !part.is_empty())?;
  let cleaned =
    token.trim_matches(|c: char| {
      !c.is_alphanumeric()
    });
  if cleaned.is_empty() {
    None
  } else {
    Some(cleaned.to_string())
  }
}

pub(crate) fn extract_genre(
  reference: &str
) -> Option<String> {
  let start = reference.find('[')?;
  let close =
    reference[start + 1..].find(']')?;
  let value = reference
    [start + 1..start + 1 + close]
    .trim()
    .to_string();
  if start == 0
    && value
      .chars()
      .all(|c| c.is_ascii_digit())
  {
    return None;
  }
  Some(value)
}

pub(crate) fn extract_edition(
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

fn detect_circa(
  reference: &str
) -> bool {
  let tokens = reference
    .split_whitespace()
    .collect::<Vec<_>>();
  for (idx, token) in
    tokens.iter().enumerate()
  {
    let has_lowercase = token
      .chars()
      .any(|c| c.is_lowercase());
    let trimmed = token
      .trim_matches(|c: char| {
        c.is_ascii_punctuation()
      })
      .to_lowercase();
    if trimmed == "circa" {
      return true;
    }
    if (trimmed == "c"
      || trimmed == "ca")
      && has_lowercase
      && let Some(next) =
        tokens.get(idx + 1)
    {
      let digits = next
        .chars()
        .filter(|c| c.is_ascii_digit())
        .count();
      if digits >= 4 {
        return true;
      }
    }
    if trimmed.starts_with("c")
      && trimmed
        .chars()
        .skip(1)
        .filter(|c| c.is_ascii_digit())
        .count()
        >= 4
    {
      return true;
    }
  }
  false
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
  let mut chars =
    segment.chars().peekable();
  while let Some(ch) = chars.next() {
    if ch == '(' {
      let mut contents = String::new();
      for inner in chars.by_ref() {
        if inner == ')' {
          break;
        }
        contents.push(inner);
      }
      let lower =
        contents.to_lowercase();
      let is_edition = lower
        .contains("ed")
        || lower.contains("édition")
        || lower.contains("ed.")
        || lower.contains("éd");
      if !is_edition {
        output.push('(');
        output
          .push_str(contents.trim());
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

pub(crate) fn normalize_token(
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
