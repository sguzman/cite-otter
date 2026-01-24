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
const LIMIT: usize = 50;

fn main() -> anyhow::Result<()> {
  let refs = extract_core_refs(
    Path::new(CORE_XML),
    LIMIT
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
  for chunk in content
    .split("<sequence>")
    .skip(1)
  {
    if let Some(end) =
      chunk.find("</sequence>")
    {
      let segment = &chunk[..end];
      let mut parts = Vec::new();
      for line in segment.lines() {
        if let Some(text) =
          extract_tag_text(line)
        {
          parts.push(text);
        }
      }
      let reference = parts
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
      if !reference.is_empty() {
        refs.push(reference);
      }
      if refs.len() >= limit {
        break;
      }
    }
  }
  Ok(refs)
}

fn extract_tag_text(
  line: &str
) -> Option<String> {
  let start = line.find('>')?;
  let end = line.rfind('<')?;
  if end <= start {
    return None;
  }
  let text = line[start + 1..end].trim();
  if text.is_empty() {
    return None;
  }
  Some(
    text
      .replace("&amp;", "&")
      .replace("&#39;", "'")
  )
}
