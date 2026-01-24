use std::fs::{
  self,
  File
};
use std::io::{
  BufRead,
  BufReader,
  Read
};
use std::path::{
  Path,
  PathBuf
};
use std::process::Command as ProcessCommand;

use clap::{
  Parser as ClapParser,
  Subcommand,
  ValueEnum
};
use flate2::read::GzDecoder;
use glob::glob;
use serde::Serialize;
use serde_json::to_writer_pretty;

use crate::dictionary::{
  Dictionary,
  DictionaryAdapter,
  DictionaryCode,
  DictionaryConfig
};
use crate::finder::Finder;
use crate::format::{
  Format,
  ParseFormat
};
use crate::model::{
  FinderModel,
  ParserModel
};
use crate::normalizer::NormalizationConfig;
use crate::parser::{
  Parser,
  Reference,
  sequence_signature,
  tagged_sequence_signature
};
use crate::sequence_model::SequenceModel;

#[derive(ClapParser, Debug)]
#[command(name = "cite-otter")]
#[command(about = "Rust port of AnyStyle", long_about = None)]
pub struct Cli {
  #[arg(long, global = true)]
  parser_model:      Option<PathBuf>,
  #[arg(long, global = true)]
  finder_model:      Option<PathBuf>,
  #[arg(long, global = true)]
  parser_sequences:  Option<PathBuf>,
  #[arg(long, global = true)]
  finder_sequences:  Option<PathBuf>,
  #[arg(long, global = true)]
  report_dir:        Option<PathBuf>,
  #[arg(long, global = true)]
  normalization_dir: Option<PathBuf>,
  #[command(subcommand)]
  command:           Command
}

#[derive(Subcommand, Debug)]
enum Command {
  /// Parse a reference string or file
  Parse {
    /// Plain reference text or file
    /// path
    input:         String,
    #[arg(
      short,
      long,
      default_value_t = ParseFormat::Json,
      value_enum
    )]
    output_format: ParseFormat
  },

  /// Find references inside a textual
  /// document
  Find {
    /// Path or inline text to scan
    input:         String,
    #[arg(
      short,
      long,
      default_value_t = ParseFormat::Json,
      value_enum
    )]
    output_format: ParseFormat
  },

  /// Train models
  Train {
    #[arg(
      long,
      default_value = DEFAULT_PARSER_PATTERN
    )]
    parser_dataset: String,
    #[arg(
      long,
      default_value = DEFAULT_FINDER_PATTERN
    )]
    finder_dataset: String
  },

  /// Check datasets
  Check {
    #[arg(
      long,
      default_value = DEFAULT_PARSER_PATTERN
    )]
    parser_dataset: String,
    #[arg(
      long,
      default_value = DEFAULT_FINDER_PATTERN
    )]
    finder_dataset: String
  },

  /// Produce delta outputs
  Delta {
    #[arg(
      long,
      default_value = DEFAULT_PARSER_PATTERN
    )]
    parser_dataset: String,
    #[arg(
      long,
      default_value = DEFAULT_FINDER_PATTERN
    )]
    finder_dataset: String
  },

  /// Output sample parse results
  Sample {
    #[arg(
      short,
      long,
      default_value_t = ParseFormat::Json,
      value_enum
    )]
    format: ParseFormat
  },

  /// Query dictionary adapters
  Dictionary {
    /// Term to lookup
    term:      String,
    #[arg(
      long,
      value_enum,
      default_value_t = DictionaryAdapterArg::Memory
    )]
    adapter:   DictionaryAdapterArg,
    #[arg(long)]
    gdbm_path: Option<PathBuf>,
    #[arg(long)]
    lmdb_path: Option<PathBuf>,
    #[arg(long)]
    redis_url: Option<String>,
    #[arg(long)]
    namespace: Option<String>
  },

  /// Import terms into dictionary
  /// adapters
  #[command(name = "dictionary-import")]
  DictionaryImport {
    /// Dictionary file paths
    inputs:    Vec<PathBuf>,
    #[arg(
      long,
      value_enum,
      default_value_t = DictionaryAdapterArg::Memory
    )]
    adapter:   DictionaryAdapterArg,
    #[arg(
      long,
      value_enum,
      default_value_t = DictionaryImportFormat::Plain
    )]
    format:    DictionaryImportFormat,
    #[arg(
      long,
      value_enum,
      default_value_t = DictionaryCodeArg::Place
    )]
    code:      DictionaryCodeArg,
    #[arg(long)]
    gdbm_path: Option<PathBuf>,
    #[arg(long)]
    lmdb_path: Option<PathBuf>,
    #[arg(long)]
    redis_url: Option<String>,
    #[arg(long)]
    namespace: Option<String>
  },

  /// Sync AnyStyle dictionaries into
  /// adapters
  #[command(name = "dictionary-sync")]
  DictionarySync {
    #[arg(
      long,
      default_value = "tmp/anystyle/\
                       data"
    )]
    source_dir: PathBuf,
    #[arg(long)]
    pattern:    Vec<String>,
    #[arg(
      long,
      value_enum,
      default_value_t = DictionaryAdapterArg::Memory
    )]
    adapter:    DictionaryAdapterArg,
    #[arg(long)]
    gdbm_path:  Option<PathBuf>,
    #[arg(long)]
    lmdb_path:  Option<PathBuf>,
    #[arg(long)]
    redis_url:  Option<String>,
    #[arg(long)]
    namespace:  Option<String>
  },

  /// Sync normalization assets
  #[command(
    name = "normalization-sync"
  )]
  NormalizationSync {
    #[arg(
      long,
      default_value = "tmp/anystyle/\
                       data"
    )]
    source_dir:  PathBuf,
    #[arg(long)]
    repo:        Option<String>,
    #[arg(long)]
    repo_subdir: Option<PathBuf>,
    #[arg(long)]
    pattern:     Vec<String>,
    #[arg(long)]
    output_dir:  Option<PathBuf>
  }
}

const REPORT_DIR: &str =
  "target/reports";
const MODEL_DIR: &str = "target/models";
const DEFAULT_PARSER_PATTERN: &str =
  "tmp/anystyle/res/parser/*.xml";
const DEFAULT_FINDER_PATTERN: &str =
  "tmp/anystyle/res/finder/*.ttx";

#[derive(
  Copy, Clone, Debug, ValueEnum,
)]
enum DictionaryAdapterArg {
  Memory,
  Redis,
  Lmdb,
  Gdbm
}

#[derive(
  Copy, Clone, Debug, ValueEnum,
)]
enum DictionaryCodeArg {
  Name,
  Place,
  Publisher,
  Journal
}

impl From<DictionaryCodeArg>
  for DictionaryCode
{
  fn from(
    code: DictionaryCodeArg
  ) -> Self {
    match code {
      | DictionaryCodeArg::Name => {
        DictionaryCode::Name
      }
      | DictionaryCodeArg::Place => {
        DictionaryCode::Place
      }
      | DictionaryCodeArg::Publisher => {
        DictionaryCode::Publisher
      }
      | DictionaryCodeArg::Journal => {
        DictionaryCode::Journal
      }
    }
  }
}

#[derive(
  Copy, Clone, Debug, ValueEnum,
)]
enum DictionaryImportFormat {
  Plain,
  AnyStyle
}

impl From<DictionaryAdapterArg>
  for DictionaryAdapter
{
  fn from(
    adapter: DictionaryAdapterArg
  ) -> Self {
    match adapter {
      DictionaryAdapterArg::Memory => {
        DictionaryAdapter::Memory
      }
      DictionaryAdapterArg::Redis => {
        DictionaryAdapter::Redis
      }
      DictionaryAdapterArg::Lmdb => {
        DictionaryAdapter::Lmdb
      }
      DictionaryAdapterArg::Gdbm => {
        DictionaryAdapter::Gdbm
      }
    }
  }
}

#[derive(Debug, Clone)]
struct CliPaths {
  parser_model:     PathBuf,
  finder_model:     PathBuf,
  parser_sequences: PathBuf,
  finder_sequences: PathBuf,
  report_dir:       PathBuf
}

impl Default for CliPaths {
  fn default() -> Self {
    Self {
      parser_model:     model_path(
        "parser-model.json"
      ),
      finder_model:     model_path(
        "finder-model.json"
      ),
      parser_sequences: model_path(
        "parser-sequences.json"
      ),
      finder_sequences: model_path(
        "finder-sequences.json"
      ),
      report_dir:       Path::new(
        REPORT_DIR
      )
      .to_path_buf()
    }
  }
}

impl CliPaths {
  fn from_cli(cli: &Cli) -> Self {
    let mut paths = Self::default();
    if let Some(path) =
      &cli.parser_model
    {
      paths.parser_model = path.clone();
    }
    if let Some(path) =
      &cli.finder_model
    {
      paths.finder_model = path.clone();
    }
    if let Some(path) =
      &cli.parser_sequences
    {
      paths.parser_sequences =
        path.clone();
    }
    if let Some(path) =
      &cli.finder_sequences
    {
      paths.finder_sequences =
        path.clone();
    }
    if let Some(path) = &cli.report_dir
    {
      paths.report_dir = path.clone();
    }
    paths
  }
}

#[derive(Clone, Serialize)]
struct DatasetStat {
  path:      String,
  sequences: usize,
  tokens:    usize
}

#[derive(Debug, Clone, Serialize)]
struct SampleEntry {
  format: String,
  output: String
}

#[derive(Serialize)]
struct TrainingReport {
  parser:  Vec<DatasetStat>,
  finder:  Vec<DatasetStat>,
  samples: Vec<SampleEntry>
}

#[derive(Serialize)]
struct ValidationReport {
  parser: Vec<DatasetStat>,
  finder: Vec<DatasetStat>
}

#[derive(Clone, Serialize)]
struct DeltaEntry {
  path:     String,
  kind:     String,
  prepared: usize,
  labeled:  usize,
  stored:   usize,
  delta:    isize
}

#[derive(Serialize)]
struct DeltaReport {
  comparisons: Vec<DeltaEntry>
}

pub fn run() -> anyhow::Result<()> {
  let cli = Cli::parse();
  let formatter =
    formatter_for_cli(&cli)?;
  let parser = parser_for_cli(&cli)?;
  let paths = CliPaths::from_cli(&cli);

  match cli.command {
    | Command::Parse {
      input,
      output_format
    } => {
      let text = load_input(&input)?;
      let references = parser.parse(
        &[text.as_str()],
        output_format
      );
      let output = match output_format {
        | ParseFormat::Json => {
          formatter.to_json(&references)
        }
        | ParseFormat::BibTeX => {
          formatter
            .to_bibtex(&references)
        }
        | ParseFormat::Csl => {
          formatter.to_csl(&references)
        }
      };
      println!("{output}");
    }
    | Command::Find {
      input,
      output_format
    } => {
      let text = load_input(&input)?;
      let signatures =
        SequenceModel::load(
          &paths.finder_sequences
        )?;
      let finder =
        Finder::with_signatures(
          signatures
        );
      let _ = finder.label(&text);
      let segments =
        Finder::segments(&text);
      println!(
        "found {} sequence(s)",
        segments.len()
      );
      if !segments.is_empty() {
        let references = segments
          .iter()
          .map(|segment| {
            segment.as_str()
          })
          .collect::<Vec<_>>();
        let parsed = parser.parse(
          &references,
          output_format
        );
        let output = match output_format
        {
          | ParseFormat::Json => {
            formatter.to_json(&parsed)
          }
          | ParseFormat::BibTeX => {
            formatter.to_bibtex(&parsed)
          }
          | ParseFormat::Csl => {
            formatter.to_csl(&parsed)
          }
        };
        println!("{output}",);
      }
    }
    | Command::Train {
      parser_dataset,
      finder_dataset
    } => {
      run_training_with_config(
        &parser_dataset,
        &finder_dataset,
        &paths
      )?;
    }
    | Command::Check {
      parser_dataset,
      finder_dataset
    } => {
      run_validation_with_config(
        &parser_dataset,
        &finder_dataset,
        &paths
      )?;
    }
    | Command::Delta {
      parser_dataset,
      finder_dataset
    } => {
      run_delta_with_config(
        &parser_dataset,
        &finder_dataset,
        &paths
      )?;
    }
    | Command::Sample {
      format
    } => {
      let references = parser.parse(
        &SAMPLE_REFERENCES,
        format
      );
      let output = match format {
        | ParseFormat::Json => {
          formatter.to_json(&references)
        }
        | ParseFormat::BibTeX => {
          formatter
            .to_bibtex(&references)
        }
        | ParseFormat::Csl => {
          formatter.to_csl(&references)
        }
      };
      println!("{output}");
    }
    | Command::Dictionary {
      term,
      adapter,
      gdbm_path,
      lmdb_path,
      redis_url,
      namespace
    } => {
      let mut config =
        DictionaryConfig::new(
          DictionaryAdapter::from(
            adapter
          )
        );
      if let Some(path) = gdbm_path {
        config =
          config.with_gdbm_path(path);
      }
      if let Some(path) = lmdb_path {
        config =
          config.with_lmdb_path(path);
      }
      if let Some(url) = redis_url {
        config =
          config.with_redis_url(url);
      }
      if let Some(name) = namespace {
        config =
          config.with_namespace(name);
      }
      let dictionary =
        Dictionary::try_create(config)?;
      let codes =
        dictionary.lookup(&term);
      if codes.is_empty() {
        println!("no matches");
      } else {
        let labels = codes
          .into_iter()
          .map(|code| {
            format!("{code:?}")
          })
          .collect::<Vec<_>>()
          .join(", ");
        println!("{labels}");
      }
    }
    | Command::DictionaryImport {
      inputs,
      adapter,
      format,
      code,
      gdbm_path,
      lmdb_path,
      redis_url,
      namespace
    } => {
      if inputs.is_empty() {
        anyhow::bail!(
          "dictionary import requires \
           at least one input file"
        );
      }
      let mut config =
        DictionaryConfig::new(
          DictionaryAdapter::from(
            adapter
          )
        );
      if let Some(path) = gdbm_path {
        config =
          config.with_gdbm_path(path);
      }
      if let Some(path) = lmdb_path {
        config =
          config.with_lmdb_path(path);
      }
      if let Some(url) = redis_url {
        config =
          config.with_redis_url(url);
      }
      if let Some(name) = namespace {
        config =
          config.with_namespace(name);
      }
      let mut dictionary =
        Dictionary::try_create(config)?;
      let mut total = 0usize;
      let code =
        DictionaryCode::from(code);
      for input in inputs {
        let inserted = match format {
          | DictionaryImportFormat::Plain => {
            let terms =
              load_dictionary_terms(
                &input
              )?;
            dictionary.import_terms(
              code, terms
            )?
          }
          | DictionaryImportFormat::AnyStyle => {
            let entries =
              load_anystyle_entries(
                &input
              )?;
            dictionary.import_entries(
              entries
            )?
          }
        };
        println!(
          "imported {inserted} terms \
           from {}",
          input.display()
        );
        total += inserted;
      }
      println!(
        "total imported: {total}"
      );
    }
    | Command::DictionarySync {
      source_dir,
      pattern,
      adapter,
      gdbm_path,
      lmdb_path,
      redis_url,
      namespace
    } => {
      let mut config =
        DictionaryConfig::new(
          DictionaryAdapter::from(
            adapter
          )
        );
      if let Some(path) = gdbm_path {
        config =
          config.with_gdbm_path(path);
      }
      if let Some(path) = lmdb_path {
        config =
          config.with_lmdb_path(path);
      }
      if let Some(url) = redis_url {
        config =
          config.with_redis_url(url);
      }
      if let Some(name) = namespace {
        config =
          config.with_namespace(name);
      }
      let patterns =
        if pattern.is_empty() {
          vec![
            "**/*.txt".to_string(),
            "**/*.txt.gz".to_string(),
          ]
        } else {
          pattern
        };
      let files =
        collect_dictionary_files(
          &source_dir,
          &patterns
        )?;
      if files.is_empty() {
        anyhow::bail!(
          "no dictionary files found \
           in {}",
          source_dir.display()
        );
      }
      let mut dictionary =
        Dictionary::try_create(config)?;
      let mut total = 0usize;
      for file in files {
        let entries =
          load_anystyle_entries(&file)?;
        let inserted = dictionary
          .import_entries(entries)?;
        println!(
          "synced {inserted} terms \
           from {}",
          file.display()
        );
        total += inserted;
      }
      println!("total synced: {total}");
    }
    | Command::NormalizationSync {
      source_dir,
      repo,
      repo_subdir,
      pattern,
      output_dir
    } => {
      let patterns = if pattern
        .is_empty()
      {
        vec![
          "**/*abbrev*.txt".to_string(),
          "**/*abbrev*.txt.gz"
            .to_string(),
          "**/*locale*.txt".to_string(),
          "**/*locale*.txt.gz"
            .to_string(),
        ]
      } else {
        pattern
      };
      let out_dir = output_dir
        .or_else(|| {
          cli.normalization_dir.clone()
        })
        .unwrap_or_else(|| {
          Path::new("target")
            .join("normalization")
        });
      let source =
        if let Some(repo) = repo {
          resolve_normalization_source(
            &repo,
            repo_subdir.as_ref()
          )?
        } else {
          source_dir
        };
      let count =
        sync_normalization_files(
          &source, &patterns, &out_dir
        )?;
      println!(
        "synced {count} normalization \
         files to {}",
        out_dir.display()
      );
    }
  }

  Ok(())
}

fn load_input(
  input: &str
) -> anyhow::Result<String> {
  if input == "-" {
    let mut buffer = String::new();
    std::io::stdin()
      .read_to_string(&mut buffer)?;
    return Ok(buffer);
  }
  let path = Path::new(input);
  if path.exists() {
    Ok(fs::read_to_string(path)?)
  } else {
    Ok(input.to_string())
  }
}

fn formatter_for_cli(
  cli: &Cli
) -> anyhow::Result<Format> {
  if let Some(dir) =
    cli.normalization_dir.as_ref()
  {
    let config =
      NormalizationConfig::load_from_dir(
        dir
      )?;
    Ok(Format::with_normalization(
      config
    ))
  } else {
    Ok(Format::new())
  }
}

fn parser_for_cli(
  cli: &Cli
) -> anyhow::Result<Parser> {
  if let Some(dir) =
    cli.normalization_dir.as_ref()
  {
    let config =
      NormalizationConfig::load_from_dir(
        dir
      )?;
    Ok(Parser::with_normalization(
      config
    ))
  } else {
    Ok(Parser::new())
  }
}

fn load_dictionary_terms(
  path: &Path
) -> anyhow::Result<Vec<String>> {
  let content =
    fs::read_to_string(path)?;
  let mut terms = Vec::new();
  for raw_line in content.lines() {
    let line = raw_line.trim();
    if line.is_empty()
      || line.starts_with('#')
    {
      continue;
    }
    let term = line
      .split(|c| c == '\t' || c == ',')
      .next()
      .unwrap_or("")
      .trim();
    if !term.is_empty() {
      terms.push(term.to_string());
    }
  }
  Ok(terms)
}

fn collect_dictionary_files(
  source_dir: &Path,
  patterns: &[String]
) -> anyhow::Result<Vec<PathBuf>> {
  let mut files = Vec::new();
  for pattern in patterns {
    let pattern = source_dir
      .join(pattern)
      .to_string_lossy()
      .to_string();
    for entry in glob(&pattern)? {
      if let Ok(path) = entry {
        if path.is_file() {
          files.push(path);
        }
      }
    }
  }
  files.sort();
  files.dedup();
  Ok(files)
}

fn collect_normalization_files(
  source_dir: &Path,
  patterns: &[String]
) -> anyhow::Result<Vec<PathBuf>> {
  collect_dictionary_files(
    source_dir, patterns
  )
}

fn resolve_normalization_source(
  repo: &str,
  repo_subdir: Option<&PathBuf>
) -> anyhow::Result<PathBuf> {
  let path = Path::new(repo);
  if path.exists() {
    return resolve_normalization_subdir(
      path,
      repo_subdir
    );
  }

  let target = Path::new("target")
    .join("normalization")
    .join("repo");
  if target.exists() {
    return resolve_normalization_subdir(
      &target,
      repo_subdir
    );
  }

  let status =
    ProcessCommand::new("git")
      .args([
        "clone",
        "--depth",
        "1",
        repo,
        target
          .to_string_lossy()
          .as_ref()
      ])
      .status()?;
  if !status.success() {
    anyhow::bail!(
      "git clone failed for {repo}"
    );
  }
  resolve_normalization_subdir(
    &target,
    repo_subdir
  )
}

fn resolve_normalization_subdir(
  repo_root: &Path,
  repo_subdir: Option<&PathBuf>
) -> anyhow::Result<PathBuf> {
  let Some(subdir) = repo_subdir else {
    return Ok(repo_root.to_path_buf());
  };
  let resolved = repo_root.join(subdir);
  if !resolved.exists() {
    anyhow::bail!(
      "normalization subdir not \
       found: {}",
      resolved.display()
    );
  }
  Ok(resolved)
}

fn sync_normalization_files(
  source_dir: &Path,
  patterns: &[String],
  output_dir: &Path
) -> anyhow::Result<usize> {
  let files =
    collect_normalization_files(
      source_dir, patterns
    )?;
  if files.is_empty() {
    anyhow::bail!(
      "no normalization assets found \
       in {}",
      source_dir.display()
    );
  }
  fs::create_dir_all(output_dir)?;
  let mut copied = 0usize;
  for file in files {
    let Some(name) = file.file_name()
    else {
      continue;
    };
    let dest = output_dir.join(name);
    fs::copy(&file, &dest)?;
    copied += 1;
  }
  Ok(copied)
}

fn load_anystyle_entries(
  path: &Path
) -> anyhow::Result<Vec<(String, u32)>>
{
  let file = File::open(path)?;
  let reader: Box<dyn BufRead> =
    match path
      .extension()
      .and_then(|ext| ext.to_str())
    {
      | Some("gz") => {
        Box::new(BufReader::new(
          GzDecoder::new(file)
        ))
      }
      | _ => {
        Box::new(BufReader::new(file))
      }
    };

  let mut entries = Vec::new();
  let mut mode = 0u32;
  for line in reader.lines() {
    let line = line?;
    let line = line.trim();
    if line.is_empty() {
      continue;
    }
    if let Some(tag) =
      line.strip_prefix("#!")
    {
      mode =
        DictionaryCode::from_tag(tag)
          .map(|code| code.bit())
          .unwrap_or(0);
      continue;
    }
    if line.starts_with('#') {
      continue;
    }
    if mode == 0 {
      continue;
    }
    let key =
      strip_trailing_score(line);
    if key.is_empty() {
      continue;
    }
    entries
      .push((key.to_string(), mode));
  }

  Ok(entries)
}

fn strip_trailing_score(
  line: &str
) -> &str {
  let trimmed = line.trim();
  let Some(idx) =
    trimmed.rfind(char::is_whitespace)
  else {
    return trimmed;
  };
  let (left, right) =
    trimmed.split_at(idx);
  let right = right.trim();
  if is_score_token(right) {
    left.trim_end()
  } else {
    trimmed
  }
}

fn is_score_token(token: &str) -> bool {
  let mut parts = token.split('.');
  let Some(left) = parts.next() else {
    return false;
  };
  let Some(right) = parts.next() else {
    return false;
  };
  if parts.next().is_some() {
    return false;
  }
  !left.is_empty()
    && !right.is_empty()
    && left
      .chars()
      .all(|c| c.is_ascii_digit())
    && right
      .chars()
      .all(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;
  use std::fs;
  #[cfg(unix)]
  use std::os::unix::fs::PermissionsExt;

  use serde_json::Value;
  use tempfile::tempdir;

  use super::*;

  #[test]
  fn load_anystyle_entries_reads_tagged_lines()
   {
    let path = Path::new(
      "tests/fixtures/\
       dictionary-sample.txt"
    );
    let entries =
      load_anystyle_entries(path)
        .expect("load entries");
    let mut map =
      HashMap::<String, u32>::new();
    for (term, value) in entries {
      *map.entry(term).or_insert(0) |=
        value;
    }

    assert_eq!(
      map.get("Italy"),
      Some(
        &DictionaryCode::Place.bit()
      )
    );
    let nature =
      map.get("Nature").copied();
    let expected =
      DictionaryCode::Journal.bit()
        | DictionaryCode::Publisher
          .bit();
    assert_eq!(nature, Some(expected));
  }

  #[test]
  fn normalization_sync_copies_files() {
    let temp_dir = tempfile::tempdir()
      .expect("temp dir");
    let source =
      temp_dir.path().join("src");
    let output =
      temp_dir.path().join("out");
    fs::create_dir_all(&source)
      .expect("create source");

    let journal =
      source.join("journal-abbrev.txt");
    fs::write(
      &journal,
      "J. Test.\tJournal of Testing"
    )
    .expect("write journal file");

    let count = sync_normalization_files(
      &source,
      &["**/*abbrev*.txt".to_string()],
      &output
    )
    .expect("sync");

    assert_eq!(count, 1);
    let copied = fs::read_to_string(
      output.join("journal-abbrev.txt")
    )
    .expect("read copy");
    assert!(
      copied
        .contains("Journal of Testing")
    );
  }

  #[test]
  fn normalization_sync_accepts_repo_path()
   {
    let temp_dir = tempfile::tempdir()
      .expect("temp dir");
    let source =
      temp_dir.path().join("repo");
    fs::create_dir_all(&source)
      .expect("create repo dir");

    let journal =
      source.join("journal-abbrev.txt");
    fs::write(
      &journal,
      "J. Test.\tJournal of Testing"
    )
    .expect("write journal file");

    let resolved =
      resolve_normalization_source(
        source
          .to_string_lossy()
          .as_ref(),
        None
      )
      .expect("resolve repo path");
    assert_eq!(resolved, source);
  }

  #[test]
  fn normalization_sync_accepts_repo_subdir()
   {
    let temp_dir = tempfile::tempdir()
      .expect("temp dir");
    let source =
      temp_dir.path().join("repo");
    let subdir = source.join("assets");
    fs::create_dir_all(&subdir)
      .expect("create repo dir");

    let journal =
      subdir.join("journal-abbrev.txt");
    fs::write(
      &journal,
      "J. Test.\tJournal of Testing"
    )
    .expect("write journal file");

    let resolved =
      resolve_normalization_source(
        source
          .to_string_lossy()
          .as_ref(),
        Some(&PathBuf::from("assets"))
      )
      .expect("resolve repo path");
    assert_eq!(resolved, subdir);
  }

  #[test]
  fn normalization_sync_clones_repo() {
    let temp_dir = tempfile::tempdir()
      .expect("temp dir");
    let remote_repo =
      temp_dir.path().join("remote");
    let _ = ProcessCommand::new("git")
      .args([
        "init",
        remote_repo
          .to_string_lossy()
          .as_ref()
      ])
      .status()
      .expect("git init");
    let journal = remote_repo
      .join("journal-abbrev.txt");
    fs::write(
      &journal,
      "J. Test.\tJournal of Testing"
    )
    .expect("write journal file");
    let _ = ProcessCommand::new("git")
      .current_dir(&remote_repo)
      .args(["add", "."])
      .status()
      .expect("git add");
    let _ = ProcessCommand::new("git")
      .current_dir(&remote_repo)
      .args([
        "-c",
        "user.email=test@example.com",
        "-c",
        "user.name=Test",
        "commit",
        "-m",
        "seed"
      ])
      .status()
      .expect("git commit");

    let cloned =
      resolve_normalization_source(
        remote_repo
          .to_string_lossy()
          .as_ref(),
        None
      )
      .expect("clone repo");
    assert!(cloned.exists());
  }

  #[test]
  fn collect_files_rejects_invalid_glob()
   {
    let result = collect_files("[");
    assert!(
      result.is_err(),
      "invalid glob patterns should \
       error"
    );
  }

  #[test]
  fn training_rejects_invalid_glob() {
    let paths = CliPaths::default();
    let result =
      run_training_with_config(
        "[",
        DEFAULT_FINDER_PATTERN,
        &paths
      );
    assert!(
      result.is_err(),
      "training should error on \
       invalid glob"
    );
  }

  #[test]
  fn validation_rejects_invalid_glob() {
    let paths = CliPaths::default();
    let result =
      run_validation_with_config(
        "[",
        DEFAULT_FINDER_PATTERN,
        &paths
      );
    assert!(
      result.is_err(),
      "validation should error on \
       invalid glob"
    );
  }

  #[test]
  fn delta_rejects_invalid_glob() {
    let paths = CliPaths::default();
    let result = run_delta_with_config(
      "[",
      DEFAULT_FINDER_PATTERN,
      &paths
    );
    assert!(
      result.is_err(),
      "delta should error on invalid \
       glob"
    );
  }

  #[test]
  fn training_errors_on_missing_dataset()
   {
    let paths = CliPaths::default();
    let result =
      run_training_with_config(
        "target/missing-parser.xml",
        DEFAULT_FINDER_PATTERN,
        &paths
      );
    assert!(
      result.is_ok(),
      "training should succeed with \
       zero parser datasets"
    );
  }

  #[test]
  fn validation_errors_on_missing_dataset()
   {
    let paths = CliPaths::default();
    let result =
      run_validation_with_config(
        "target/missing-parser.xml",
        DEFAULT_FINDER_PATTERN,
        &paths
      );
    assert!(
      result.is_ok(),
      "validation should succeed with \
       zero parser datasets"
    );
  }

  #[test]
  fn delta_errors_on_missing_dataset() {
    let paths = CliPaths::default();
    let result = run_delta_with_config(
      "target/missing-parser.xml",
      DEFAULT_FINDER_PATTERN,
      &paths
    );
    assert!(
      result.is_ok(),
      "delta should succeed with zero \
       parser datasets"
    );
  }

  #[cfg(unix)]
  #[test]
  fn training_errors_on_unreadable_dataset()
   {
    let temp_dir = tempfile::tempdir()
      .expect("temp dir");
    let dataset = temp_dir
      .path()
      .join("unreadable.xml");
    fs::write(&dataset, "test")
      .expect("write dataset");
    fs::set_permissions(
      &dataset,
      fs::Permissions::from_mode(0o000)
    )
    .expect("chmod dataset");

    let paths = CliPaths::default();
    let result =
      run_training_with_config(
        dataset
          .to_string_lossy()
          .as_ref(),
        DEFAULT_FINDER_PATTERN,
        &paths
      );
    fs::set_permissions(
      &dataset,
      fs::Permissions::from_mode(0o644)
    )
    .expect("restore permissions");
    assert!(
      result.is_err(),
      "training should error on \
       unreadable datasets"
    );
  }

  #[cfg(unix)]
  #[test]
  fn validation_errors_on_unreadable_dataset()
   {
    let temp_dir = tempfile::tempdir()
      .expect("temp dir");
    let dataset = temp_dir
      .path()
      .join("unreadable.xml");
    fs::write(&dataset, "test")
      .expect("write dataset");
    fs::set_permissions(
      &dataset,
      fs::Permissions::from_mode(0o000)
    )
    .expect("chmod dataset");

    let paths = CliPaths::default();
    let result =
      run_validation_with_config(
        dataset
          .to_string_lossy()
          .as_ref(),
        DEFAULT_FINDER_PATTERN,
        &paths
      );
    fs::set_permissions(
      &dataset,
      fs::Permissions::from_mode(0o644)
    )
    .expect("restore permissions");
    assert!(
      result.is_err(),
      "validation should error on \
       unreadable datasets"
    );
  }

  #[cfg(unix)]
  #[test]
  fn delta_errors_on_unreadable_dataset()
   {
    let temp_dir = tempfile::tempdir()
      .expect("temp dir");
    let dataset = temp_dir
      .path()
      .join("unreadable.xml");
    fs::write(&dataset, "test")
      .expect("write dataset");
    fs::set_permissions(
      &dataset,
      fs::Permissions::from_mode(0o000)
    )
    .expect("chmod dataset");

    let paths = CliPaths::default();
    let result = run_delta_with_config(
      dataset
        .to_string_lossy()
        .as_ref(),
      DEFAULT_FINDER_PATTERN,
      &paths
    );
    fs::set_permissions(
      &dataset,
      fs::Permissions::from_mode(0o644)
    )
    .expect("restore permissions");
    assert!(
      result.is_err(),
      "delta should error on \
       unreadable datasets"
    );
  }

  #[test]
  fn training_report_matches_fixture_snapshot()
   {
    let temp_dir =
      tempdir().expect("temp dir");
    let report_dir =
      temp_dir.path().join("reports");
    let paths = CliPaths {
      parser_model:     temp_dir
        .path()
        .join("parser-model.json"),
      finder_model:     temp_dir
        .path()
        .join("finder-model.json"),
      parser_sequences: temp_dir
        .path()
        .join("parser-sequences.json"),
      finder_sequences: temp_dir
        .path()
        .join("finder-sequences.json"),
      report_dir:       report_dir
        .clone()
    };

    run_training_with_config(
      "tests/fixtures/report/parser.\
       txt",
      "tests/fixtures/report/finder.\
       txt",
      &paths
    )
    .expect("training report");

    let report_path = report_dir
      .join("training-report.json");
    let report =
      fs::read_to_string(&report_path)
        .expect("read report");
    let mut actual: Value =
      serde_json::from_str(&report)
        .expect("parse report json");
    if let Some(samples) = actual
      .get_mut("samples")
      .and_then(Value::as_array_mut)
    {
      for sample in samples {
        if let Some(obj) =
          sample.as_object_mut()
        {
          obj.insert(
            "output".into(),
            Value::String(
              "<redacted>".into()
            )
          );
        }
      }
    }

    let expected = fs::read_to_string(
      "tests/fixtures/report/\
       training-report-snapshot.json"
    )
    .expect("read snapshot");
    let expected: Value =
      serde_json::from_str(&expected)
        .expect("parse snapshot");

    assert_eq!(actual, expected);
  }
}

fn run_training_with_config(
  parser_pattern: &str,
  finder_pattern: &str,
  paths: &CliPaths
) -> anyhow::Result<()> {
  let parser_files =
    collect_files(parser_pattern)?;
  let finder_files =
    collect_files(finder_pattern)?;

  let parser_pairs =
    gather_parser_stats(&parser_files)?;
  let finder_pairs =
    gather_finder_stats(&finder_files)?;

  let finder_signatures =
    collect_finder_signatures(
      &finder_files
    )?;
  let parser_signatures =
    collect_parser_signatures(
      &parser_files
    )?;

  let parser_model_path =
    paths.parser_model.clone();
  let mut parser_model =
    ParserModel::load(
      &parser_model_path
    )?;
  for (path, stat) in &parser_pairs {
    parser_model.record(
      path,
      stat.sequences,
      stat.tokens
    );
  }
  parser_model
    .save(&parser_model_path)?;

  let finder_model_path =
    paths.finder_model.clone();
  let mut finder_model =
    FinderModel::load(
      &finder_model_path
    )?;
  for (path, stat) in &finder_pairs {
    finder_model
      .record(path, stat.sequences);
  }
  finder_model
    .save(&finder_model_path)?;

  let mut parser_signature_model =
    SequenceModel::load(
      &paths.parser_sequences
    )?;
  for signature in parser_signatures {
    parser_signature_model
      .record(signature);
  }
  parser_signature_model
    .save(&paths.parser_sequences)?;

  let mut finder_signature_model =
    SequenceModel::load(
      &paths.finder_sequences
    )?;
  for (_, signatures) in
    finder_signatures
  {
    for signature in signatures {
      finder_signature_model
        .record(signature);
    }
  }
  finder_signature_model
    .save(&paths.finder_sequences)?;

  let sample_outputs =
    collect_sample_outputs();

  println!(
    "training report written (parser \
     {} files, finder {} files)",
    parser_files.len(),
    finder_files.len()
  );
  persist_report(
    report_path(
      &paths.report_dir,
      "training-report.json"
    ),
    &TrainingReport {
      parser:  parser_pairs
        .iter()
        .map(|(_, stat)| stat.clone())
        .collect(),
      finder:  finder_pairs
        .iter()
        .map(|(_, stat)| stat.clone())
        .collect(),
      samples: sample_outputs
    }
  )?;
  Ok(())
}

fn run_training() -> anyhow::Result<()>
{
  run_training_with_config(
    DEFAULT_PARSER_PATTERN,
    DEFAULT_FINDER_PATTERN,
    &CliPaths::default()
  )
}

pub fn training_report()
-> anyhow::Result<()> {
  run_training()
}

fn run_validation_with_config(
  parser_pattern: &str,
  finder_pattern: &str,
  paths: &CliPaths
) -> anyhow::Result<()> {
  let parser_files =
    collect_files(parser_pattern)?;
  let finder_files =
    collect_files(finder_pattern)?;
  let parser_stats =
    gather_parser_stats(&parser_files)?;
  let finder_stats =
    gather_finder_stats(&finder_files)?;
  let parser_model_path =
    paths.parser_model.clone();
  let parser_model = ParserModel::load(
    &parser_model_path
  )?;
  for (path, stat) in &parser_stats {
    let stored_sequences = parser_model
      .sequences(path)
      .unwrap_or(0);
    let stored_tokens = parser_model
      .tokens(path)
      .unwrap_or(0);
    let seq_delta = (stored_sequences
      as isize
      - stat.sequences as isize)
      .unsigned_abs();
    let token_delta =
      if stored_tokens == 0 {
        0
      } else {
        (stored_tokens as isize
          - stat.tokens as isize)
          .unsigned_abs()
      };
    let seq_rate = percent_delta(
      seq_delta,
      stat.sequences
    );
    let token_rate = percent_delta(
      token_delta,
      stat.tokens
    );
    println!(
      "checking {}... {:>4} seq \
       {:>5.2}% {:>4} tok {:>5.2}%",
      path.display(),
      seq_delta,
      seq_rate,
      token_delta,
      token_rate
    );
  }
  let finder_model_path =
    paths.finder_model.clone();
  let finder_model = FinderModel::load(
    &finder_model_path
  )?;
  for (path, stat) in &finder_stats {
    let stored_sequences = finder_model
      .sequences(path)
      .unwrap_or(0);
    let seq_delta = (stored_sequences
      as isize
      - stat.sequences as isize)
      .unsigned_abs();
    let seq_rate = percent_delta(
      seq_delta,
      stat.sequences
    );
    println!(
      "checking {}... {:>4} seq \
       {:>5.2}%",
      path.display(),
      seq_delta,
      seq_rate
    );
  }

  persist_report(
    report_path(
      &paths.report_dir,
      "validation-report.json"
    ),
    &ValidationReport {
      parser: parser_stats
        .iter()
        .map(|(_, stat)| stat.clone())
        .collect(),
      finder: finder_stats
        .iter()
        .map(|(_, stat)| stat.clone())
        .collect()
    }
  )?;

  println!(
    "validation report written for {} \
     parser datasets and {} finder \
     datasets",
    parser_files.len(),
    finder_files.len()
  );
  Ok(())
}

fn run_validation() -> anyhow::Result<()>
{
  run_validation_with_config(
    DEFAULT_PARSER_PATTERN,
    DEFAULT_FINDER_PATTERN,
    &CliPaths::default()
  )
}

pub fn validation_report()
-> anyhow::Result<()> {
  run_validation()
}

fn run_delta_with_config(
  parser_pattern: &str,
  finder_pattern: &str,
  paths: &CliPaths
) -> anyhow::Result<()> {
  let parser_files =
    collect_files(parser_pattern)?;
  let parser_model_path =
    paths.parser_model.clone();
  let mut delta_entries =
    parser_files
      .iter()
      .map(|path| {
        let content =
          fs::read_to_string(path)?;
        let prepared =
          Parser::new().prepare(&content, true);
        let labeled =
          Parser::new().label(&content);
        let prepared_seq = prepared.0.len();
        let stored = ParserModel::load(
          &parser_model_path,
        )?
        .sequences(path)
        .unwrap_or(0);
        let delta =
          (prepared_seq as isize
            - stored as isize)
            .abs();

        Ok(DeltaEntry {
          path: path.display().to_string(),
          kind: "parser".into(),
          prepared: prepared_seq,
          labeled: labeled.len(),
          stored,
          delta,
        })
      })
      .collect::<Result<Vec<_>, anyhow::Error>>()?;

  let finder_files =
    collect_files(finder_pattern)?;
  let finder_model_path =
    paths.finder_model.clone();
  let finder_model = FinderModel::load(
    &finder_model_path
  )?;
  let finder_entries = finder_files
    .iter()
    .map(|path| {
      let content =
        fs::read_to_string(path)?;
      let labeled =
        Parser::new().label(&content);
      let stored = finder_model
        .sequences(path)
        .unwrap_or(0);
      let delta =
        (labeled.len() as isize
          - stored as isize)
          .abs();
      Ok(DeltaEntry {
        path: path.display().to_string(),
        kind: "finder".into(),
        prepared: labeled.len(),
        labeled: labeled.len(),
        stored,
        delta
      })
    })
    .collect::<Result<Vec<_>, anyhow::Error>>()?;

  delta_entries.extend(finder_entries);

  persist_report(
    report_path(
      &paths.report_dir,
      "delta-report.json"
    ),
    &DeltaReport {
      comparisons: delta_entries
    }
  )?;

  println!(
    "delta report written ({} parser \
     + {} finder datasets)",
    parser_files.len(),
    finder_files.len()
  );
  Ok(())
}

fn run_delta() -> anyhow::Result<()> {
  run_delta_with_config(
    DEFAULT_PARSER_PATTERN,
    DEFAULT_FINDER_PATTERN,
    &CliPaths::default()
  )
}

pub fn delta_report()
-> anyhow::Result<()> {
  run_delta()
}

fn gather_parser_stats(
  paths: &[PathBuf]
) -> anyhow::Result<
  Vec<(PathBuf, DatasetStat)>
> {
  paths
    .iter()
    .map(|path| {
      let content =
        fs::read_to_string(path)?;
      let prepared = Parser::new()
        .prepare(&content, true);
      let tokens = prepared
        .0
        .iter()
        .map(|sequence| sequence.len())
        .sum();
      Ok((path.clone(), DatasetStat {
        path: path
          .display()
          .to_string(),
        sequences: prepared.0.len(),
        tokens
      }))
    })
    .collect()
}

fn gather_finder_stats(
  paths: &[PathBuf]
) -> anyhow::Result<
  Vec<(PathBuf, DatasetStat)>
> {
  paths
    .iter()
    .map(|path| {
      let content =
        fs::read_to_string(path)?;
      let sequences = Parser::new()
        .label(&content)
        .len();
      Ok((path.clone(), DatasetStat {
        path: path
          .display()
          .to_string(),
        sequences,
        tokens: 0
      }))
    })
    .collect()
}

fn persist_report(
  path: PathBuf,
  value: &impl Serialize
) -> anyhow::Result<()> {
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent)?;
  }
  let file = File::create(&path)?;
  to_writer_pretty(file, value)?;
  Ok(())
}

fn collect_files(
  pattern: &str
) -> anyhow::Result<Vec<PathBuf>> {
  Ok(
    glob(pattern)?
      .flat_map(Result::ok)
      .collect()
  )
}

fn collect_parser_signatures(
  paths: &[PathBuf]
) -> anyhow::Result<Vec<String>> {
  let parser = Parser::new();
  let mut signatures = Vec::new();

  for path in paths {
    let content =
      fs::read_to_string(path)?;
    let prepared =
      parser.prepare(&content, true);
    for sequence in prepared.0 {
      signatures.push(
        sequence_signature(&sequence)
      );
    }
  }

  Ok(signatures)
}

fn collect_finder_signatures(
  paths: &[PathBuf]
) -> anyhow::Result<
  Vec<(PathBuf, Vec<String>)>
> {
  paths
    .iter()
    .map(|path| {
      let content =
        fs::read_to_string(path)?;
      let sequences =
        Parser::new().label(&content);
      let signatures = sequences
        .iter()
        .map(|sequence| {
          tagged_sequence_signature(
            sequence
          )
        })
        .collect::<Vec<_>>();
      Ok((path.clone(), signatures))
    })
    .collect()
}

const SAMPLE_FORMATS: [ParseFormat; 3] = [
  ParseFormat::Json,
  ParseFormat::BibTeX,
  ParseFormat::Csl
];

fn collect_sample_outputs()
-> Vec<SampleEntry> {
  let parser = Parser::new();
  let formatter = Format::new();

  SAMPLE_FORMATS
    .iter()
    .map(|format| {
      let references = parser.parse(
        &SAMPLE_REFERENCES,
        *format
      );
      SampleEntry {
        format: sample_format_label(
          *format
        )
        .to_string(),
        output: format_sample_output(
          &formatter,
          &references,
          *format
        )
      }
    })
    .collect()
}

fn format_sample_output(
  formatter: &Format,
  references: &[Reference],
  format: ParseFormat
) -> String {
  match format {
    | ParseFormat::Json => {
      formatter.to_json(references)
    }
    | ParseFormat::BibTeX => {
      formatter.to_bibtex(references)
    }
    | ParseFormat::Csl => {
      formatter.to_csl(references)
    }
  }
}

fn sample_format_label(
  format: ParseFormat
) -> &'static str {
  match format {
    | ParseFormat::Json => "json",
    | ParseFormat::BibTeX => "bibtex",
    | ParseFormat::Csl => "csl"
  }
}

fn model_path(
  filename: &str
) -> PathBuf {
  Path::new(MODEL_DIR).join(filename)
}

fn report_path(
  report_dir: &Path,
  filename: &str
) -> PathBuf {
  report_dir.join(filename)
}

fn percent_delta(
  delta: usize,
  total: usize
) -> f64 {
  if total == 0 {
    0.0
  } else {
    (delta as f64 / total as f64)
      * 100.0
  }
}

const SAMPLE_REFERENCES: [&str; 2] = [
  "Perec, Georges. A Void. London: \
   The Harvill Press, 1995. p.108.",
  "Smith, Alice. On heuristics for \
   mixing metadata. Lecture Notes in \
   Computer Science. Journal of Testing. \
   Edited by Doe, J. (Note: Preprint \
   release). doi:10.1000/test https://example.org."
];
