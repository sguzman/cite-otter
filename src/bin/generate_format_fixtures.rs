use std::fs;
use std::path::Path;

use cite_otter::format::{
  Format,
  ParseFormat
};
use cite_otter::parser::Parser;

const CORE_XML: &str =
  "tmp/anystyle/res/parser/core.xml";
const OUT_DIR: &str = "tests/fixtures/format";
const LIMIT: usize = 200;

fn main() -> anyhow::Result<()> {
  let limit = std::env::var(
    "CITE_OTTER_CORE_LIMIT"
  )
  .ok()
  .and_then(|value| value.parse().ok())
  .unwrap_or(LIMIT);

  let refs = extract_core_refs(
    Path::new(CORE_XML),
    limit
  )?;
  if refs.is_empty() {
    anyhow::bail!(
      "no references extracted from \
       {CORE_XML}"
    );
  }

  fs::create_dir_all(OUT_DIR)?;
  let refs_path =
    Path::new(OUT_DIR).join("core-refs.txt");
  fs::write(
    &refs_path,
    refs.join("\n")
  )?;

  let ref_slices =
    refs.iter().map(|line| line.as_str()).collect::<Vec<_>>();
  let parser = Parser::new();
  let parsed = parser.parse(
    &ref_slices,
    ParseFormat::Json
  );
  let formatter = Format::new();

  let csl =
    formatter.to_csl(&parsed);
  let csl_path =
    Path::new(OUT_DIR).join("core-csl.txt");
  fs::write(csl_path, csl)?;

  let bibtex =
    formatter.to_bibtex(&parsed);
  let bibtex_path =
    Path::new(OUT_DIR).join("core-bibtex.txt");
  fs::write(bibtex_path, bibtex)?;

  Ok(())
}

fn extract_core_refs(
  path: &Path,
  limit: usize
) -> anyhow::Result<Vec<String>> {
  let content = fs::read_to_string(path)?;
  let mut refs = Vec::new();
  let mut cursor = content.as_str();
  while let Some(start) =
    cursor.find("<sequence>")
  {
    cursor = &cursor[start + 10..];
    let Some(end) =
      cursor.find("</sequence>")
    else {
      break;
    };
    let segment = &cursor[..end];
    let mut parts = Vec::new();
    for line in segment.lines() {
      if let Some((tag, text)) =
        extract_tag_line(line)
      {
        parts.push((tag, text));
      }
    }
    let reference =
      normalize_reference(render_reference(&parts));
    if !reference.is_empty() {
      refs.push(reference);
    }
    if refs.len() >= limit {
      break;
    }
    cursor = &cursor[end + 11..];
  }
  Ok(refs)
}

fn extract_tag_line(
  line: &str
) -> Option<(String, String)> {
  let trimmed = line.trim();
  if !trimmed.starts_with('<') {
    return None;
  }
  let tag_start = trimmed.find('<')? + 1;
  let tag_end = trimmed.find('>')?;
  let tag = trimmed[tag_start..tag_end]
    .trim()
    .trim_start_matches('/')
    .to_string();
  if tag.is_empty() {
    return None;
  }
  let close_tag = format!("</{tag}>");
  let close_idx = trimmed.rfind(&close_tag)?;
  if close_idx <= tag_end {
    return None;
  }
  let text = trimmed[tag_end + 1..close_idx]
    .trim();
  if text.is_empty() {
    return None;
  }
  Some((tag, decode_entities(text)))
}

fn decode_entities(
  value: &str
) -> String {
  value
    .replace("&amp;", "&")
    .replace("&#39;", "'")
    .replace("&quot;", "\"")
    .replace("&apos;", "'")
    .replace("&nbsp;", " ")
}

fn render_reference(
  parts: &[(String, String)]
) -> String {
  let mut output = String::new();
  let mut previous = String::new();
  for (tag, text) in parts {
    let text = normalize_tag_text(tag, text);
    let text = text.trim();
    if text.is_empty() {
      continue;
    }
    if output.is_empty() {
      output.push_str(text);
      previous = text.to_string();
      continue;
    }
    let needs_space = !ends_with_punct(&previous)
      && !starts_with_punct(text);
    if needs_space {
      output.push(' ');
    } else if !output.ends_with(' ')
      && !starts_with_punct(text)
    {
      output.push(' ');
    }
    output.push_str(text);
    previous = text.to_string();
  }
  output
}

fn normalize_tag_text(
  tag: &str,
  text: &str
) -> String {
  let trimmed = text.trim();
  if trimmed.is_empty() {
    return String::new();
  }
  let needs_period = matches!(
    tag,
    "author"
      | "title"
      | "location"
      | "publisher"
      | "container-title"
      | "collection-title"
      | "editor"
      | "translator"
      | "note"
      | "date"
      | "pages"
  );
  if needs_period && !ends_with_punct(trimmed)
  {
    format!("{trimmed}.")
  } else {
    trimmed.to_string()
  }
}

fn ends_with_punct(
  value: &str
) -> bool {
  value
    .trim_end()
    .ends_with(|c: char| {
      matches!(c, '.' | ',' | ';' | ':' | ')')
    })
}

fn starts_with_punct(
  value: &str
) -> bool {
  value
    .trim_start()
    .starts_with(|c: char| {
      matches!(c, '.' | ',' | ';' | ':' | ')')
    })
}

fn normalize_reference(
  raw: String
) -> String {
  let mut reference =
    raw.replace('\u{a0}', " ");
  reference = reference
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ");
  for (from, to) in [
    (" ,", ","),
    (" .", "."),
    (" ;", ";"),
    (" :", ":"),
    (" )", ")"),
    ("( ", "(")
  ] {
    reference = reference.replace(from, to);
  }
  reference.trim().to_string()
}
