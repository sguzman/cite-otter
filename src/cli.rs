use std::fs::{
  self,
  File
};
use std::path::{
  Path,
  PathBuf
};

use clap::{
  Parser as ClapParser,
  Subcommand
};
use glob::glob;
use serde::Serialize;
use serde_json::to_writer_pretty;

use crate::finder::Finder;
use crate::format::{
  Format,
  ParseFormat
};
use crate::model::{
  FinderModel,
  ParserModel
};
use crate::parser::Parser;

#[derive(ClapParser, Debug)]
#[command(name = "cite-otter")]
#[command(about = "Rust port of AnyStyle", long_about = None)]
pub struct Cli {
  #[command(subcommand)]
  command: Command
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
    input: String
  },

  /// Train models
  Train,

  /// Check datasets
  Check,

  /// Produce delta outputs
  Delta
}

const REPORT_DIR: &str =
  "target/reports";
const MODEL_DIR: &str = "target/models";

#[derive(Clone, Serialize)]
struct DatasetStat {
  path:      String,
  sequences: usize,
  tokens:    usize
}

#[derive(Serialize)]
struct TrainingReport {
  parser: Vec<DatasetStat>,
  finder: Vec<DatasetStat>
}

#[derive(Serialize)]
struct ValidationReport {
  parser: Vec<DatasetStat>
}

#[derive(Clone, Serialize)]
struct DeltaEntry {
  path:     String,
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
  let formatter = Format::new();

  match cli.command {
    | Command::Parse {
      input,
      output_format
    } => {
      let text = load_input(&input)?;
      let parser = Parser::new();
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
      input
    } => {
      let text = load_input(&input)?;
      let finder = Finder::new();
      let _ = finder.label(&text);
      let segments =
        Finder::segments(&text);
      println!(
        "found {} sequence(s)",
        segments.len()
      );
      if !segments.is_empty() {
        let parser = Parser::new();
        let references = segments
          .iter()
          .map(|segment| {
            segment.as_str()
          })
          .collect::<Vec<_>>();
        let parsed = parser.parse(
          &references,
          ParseFormat::Json
        );
        println!(
          "{}",
          formatter.to_json(&parsed)
        );
      }
    }
    | Command::Train => {
      run_training()?;
    }
    | Command::Check => {
      run_validation()?;
    }
    | Command::Delta => {
      run_delta()?;
    }
  }

  Ok(())
}

fn load_input(
  input: &str
) -> anyhow::Result<String> {
  let path = Path::new(input);
  if path.exists() {
    Ok(fs::read_to_string(path)?)
  } else {
    Ok(input.to_string())
  }
}

fn run_training() -> anyhow::Result<()>
{
  let parser_files = collect_files(
    "tmp/anystyle/res/parser/*.xml"
  )?;
  let finder_files = collect_files(
    "tmp/anystyle/res/finder/*.ttx"
  )?;

  let parser_pairs =
    gather_parser_stats(&parser_files)?;
  let finder_pairs =
    gather_finder_stats(&finder_files)?;

  persist_report(
    Path::new(REPORT_DIR)
      .join("training-report.json"),
    &TrainingReport {
      parser: parser_pairs
        .iter()
        .map(|(_, stat)| stat.clone())
        .collect(),
      finder: finder_pairs
        .iter()
        .map(|(_, stat)| stat.clone())
        .collect()
    }
  )?;

  let parser_model_path =
    model_path("parser-model.json");
  let mut parser_model =
    ParserModel::load(
      &parser_model_path
    )?;
  for (path, stat) in &parser_pairs {
    parser_model
      .record(path, stat.sequences);
  }
  parser_model
    .save(&parser_model_path)?;

  let finder_model_path =
    model_path("finder-model.json");
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

  println!(
    "training report written (parser \
     {} files, finder {} files)",
    parser_files.len(),
    finder_files.len()
  );
  Ok(())
}

pub fn training_report()
-> anyhow::Result<()> {
  run_training()
}

fn run_validation() -> anyhow::Result<()>
{
  let parser_files = collect_files(
    "tmp/anystyle/res/parser/*.xml"
  )?;
  let parser_stats =
    gather_parser_stats(&parser_files)?;
  let parser_model_path =
    model_path("parser-model.json");
  let parser_model = ParserModel::load(
    &parser_model_path
  )?;
  for (path, stat) in &parser_stats {
    if let Some(stored) =
      parser_model.sequences(path)
      && stored != stat.sequences
    {
      println!(
        "  validation mismatch {}: \
         stored {} vs current {}",
        path.display(),
        stored,
        stat.sequences
      );
    }
  }

  persist_report(
    Path::new(REPORT_DIR)
      .join("validation-report.json"),
    &ValidationReport {
      parser: parser_stats
        .iter()
        .map(|(_, stat)| stat.clone())
        .collect()
    }
  )?;

  println!(
    "validation report written for {} \
     datasets",
    parser_files.len()
  );
  Ok(())
}

pub fn validation_report()
-> anyhow::Result<()> {
  run_validation()
}

fn run_delta() -> anyhow::Result<()> {
  let parser_files = collect_files(
    "tmp/anystyle/res/parser/*.xml"
  )?;
  let parser_model_path =
    model_path("parser-model.json");
  let delta_entries =
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
          prepared: prepared_seq,
          labeled: labeled.len(),
          stored,
          delta,
        })
      })
      .collect::<Result<Vec<_>, anyhow::Error>>()?;

  let delta_count = delta_entries.len();
  persist_report(
    Path::new(REPORT_DIR)
      .join("delta-report.json"),
    &DeltaReport {
      comparisons: delta_entries
    }
  )?;

  println!(
    "delta report written ({} \
     datasets)",
    delta_count
  );
  Ok(())
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
  fs::create_dir_all(REPORT_DIR)?;
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

fn model_path(
  filename: &str
) -> PathBuf {
  Path::new(MODEL_DIR).join(filename)
}
