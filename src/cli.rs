use std::fs;
use std::path::{
  Path,
  PathBuf
};

use clap::{
  Parser as ClapParser,
  Subcommand
};
use glob::glob;

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

  println!(
    "training stub: parser datasets \
     {} files, finder sources {} files",
    parser_files.len(),
    finder_files.len()
  );

  for path in parser_files {
    let content =
      fs::read_to_string(&path)?;
    let sequences = Parser::new()
      .prepare(&content, true)
      .0
      .len();
    println!(
      "  parser dataset {} has {} \
       sequences",
      path.display(),
      sequences
    );
  }

  for path in finder_files {
    let content =
      fs::read_to_string(&path)?;
    let sequences = Parser::new()
      .label(&content)
      .len();
    println!(
      "  finder dataset {} yields {} \
       sequences",
      path.display(),
      sequences
    );
  }

  Ok(())
}

fn run_validation() -> anyhow::Result<()>
{
  let parser_files = collect_files(
    "tmp/anystyle/res/parser/*.xml"
  )?;
  println!(
    "validation stub: running on {} \
     parser datasets",
    parser_files.len()
  );

  for path in parser_files {
    let content =
      fs::read_to_string(&path)?;
    let labeled =
      Parser::new().label(&content);
    println!(
      "  {} -> {} sequences labeled",
      path.display(),
      labeled.len()
    );
  }

  Ok(())
}

fn run_delta() -> anyhow::Result<()> {
  let parser_files = collect_files(
    "tmp/anystyle/res/parser/*.xml"
  )?;
  println!(
    "delta stub: comparing {} \
     datasets against new labels",
    parser_files.len()
  );

  for path in parser_files {
    let content =
      fs::read_to_string(&path)?;
    let prepared = Parser::new()
      .prepare(&content, true);
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

    println!(
      "  {} -> delta {} sequences",
      path.display(),
      delta
    );
  }

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
