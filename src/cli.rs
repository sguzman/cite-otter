use std::fs;
use std::path::Path;

use clap::{
  Parser as ClapParser,
  Subcommand
};

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
      println!(
        "training not implemented yet"
      );
    }
    | Command::Check => {
      println!(
        "validation not implemented \
         yet"
      );
    }
    | Command::Delta => {
      println!(
        "delta reporting not \
         implemented yet"
      );
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
