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
    input: String
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

#[derive(Serialize)]
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

#[derive(Serialize)]
struct DeltaEntry {
  path:     String,
  prepared: usize,
  labeled:  usize,
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
      input
    } => {
      let text = load_input(&input)?;
      let parser = Parser::new();
      let references = parser.parse(
        &[text.as_str()],
        ParseFormat::Json
      );
      println!(
        "{}",
        formatter.to_json(&references)
      );
    }
    | Command::Find {
      input
    } => {
      let text = load_input(&input)?;
      let finder = Finder::new();
      let sequences =
        finder.label(&text);
      println!(
        "found {} sequence(s)",
        sequences.len()
      );
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

  let parser_stats =
    gather_parser_stats(&parser_files)?;
  let finder_stats =
    gather_finder_stats(&finder_files)?;

  persist_report(
    Path::new(REPORT_DIR)
      .join("training-report.json"),
    &TrainingReport {
      parser: parser_stats,
      finder: finder_stats
    }
  )?;

  println!(
    "training report written (parser \
     {} files, finder {} files)",
    parser_files.len(),
    finder_files.len()
  );
  Ok(())
}

fn run_validation() -> anyhow::Result<()>
{
  let parser_files = collect_files(
    "tmp/anystyle/res/parser/*.xml"
  )?;
  let parser_stats =
    gather_parser_stats(&parser_files)?;

  persist_report(
    Path::new(REPORT_DIR)
      .join("validation-report.json"),
    &ValidationReport {
      parser: parser_stats
    }
  )?;

  println!(
    "validation report written for {} \
     datasets",
    parser_files.len()
  );
  Ok(())
}

fn run_delta() -> anyhow::Result<()> {
  let parser_files = collect_files(
    "tmp/anystyle/res/parser/*.xml"
  )?;
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
        let delta = if prepared.0.len()
          == labeled.len()
        {
          0
        } else {
          (prepared.0.len() as isize
            - labeled.len() as isize)
            .abs()
        };

        Ok(DeltaEntry {
          path: path.display().to_string(),
          prepared: prepared.0.len(),
          labeled: labeled.len(),
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

fn gather_parser_stats(
  paths: &[PathBuf]
) -> anyhow::Result<Vec<DatasetStat>> {
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
      Ok(DatasetStat {
        path: path
          .display()
          .to_string(),
        sequences: prepared.0.len(),
        tokens
      })
    })
    .collect()
}

fn gather_finder_stats(
  paths: &[PathBuf]
) -> anyhow::Result<Vec<DatasetStat>> {
  paths
    .iter()
    .map(|path| {
      let content =
        fs::read_to_string(path)?;
      let sequences = Parser::new()
        .label(&content)
        .len();
      Ok(DatasetStat {
        path: path
          .display()
          .to_string(),
        sequences,
        tokens: 0
      })
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
