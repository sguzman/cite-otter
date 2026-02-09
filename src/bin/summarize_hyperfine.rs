use std::collections::BTreeMap;
use std::path::{
  Path,
  PathBuf
};
use std::{
  env,
  fs
};

use anyhow::{
  Context,
  Result,
  bail
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct HyperfineResult {
  command: String,
  mean:    f64,
  stddev:  f64
}

#[derive(Debug, Deserialize)]
struct HyperfineReport {
  results: Vec<HyperfineResult>
}

#[derive(Debug, Default)]
struct SpeedPair {
  ruby: Option<f64>,
  rust: Option<f64>
}

fn main() -> Result<()> {
  let mut args = env::args().skip(1);
  let input = args.next().context(
    "usage: summarize_hyperfine \
     <input-json> [output-md]"
  )?;
  let input_path = PathBuf::from(input);
  let output_path =
    if let Some(output) = args.next() {
      PathBuf::from(output)
    } else {
      default_output_path(&input_path)
    };
  if args.next().is_some() {
    bail!(
      "usage: summarize_hyperfine \
       <input-json> [output-md]"
    );
  }

  let data =
    fs::read_to_string(&input_path)
      .with_context(|| {
        format!(
          "read hyperfine report {}",
          input_path.display()
        )
      })?;
  let report: HyperfineReport =
    serde_json::from_str(&data)
      .with_context(|| {
        format!(
          "parse hyperfine report {}",
          input_path.display()
        )
      })?;
  let rendered =
    render_report(&report, &input_path);
  if let Some(parent) =
    output_path.parent()
  {
    fs::create_dir_all(parent)
      .with_context(|| {
        format!(
          "create summary directory {}",
          parent.display()
        )
      })?;
  }
  fs::write(&output_path, rendered)
    .with_context(|| {
      format!(
        "write hyperfine summary {}",
        output_path.display()
      )
    })?;
  println!(
    "wrote {}",
    output_path.display()
  );
  Ok(())
}

fn default_output_path(
  input: &Path
) -> PathBuf {
  let parent = input
    .parent()
    .unwrap_or_else(|| Path::new("."));
  let stem = input
    .file_stem()
    .and_then(|s| s.to_str())
    .unwrap_or("hyperfine");
  parent
    .join(format!("{stem}-summary.md"))
}

fn parse_impl_and_task(
  command: &str
) -> Option<(&str, &str)> {
  let (impl_name, task) =
    command.split_once(':')?;
  match impl_name {
    | "ruby" | "rust" => {
      Some((impl_name, task))
    }
    | _ => None
  }
}

fn render_report(
  report: &HyperfineReport,
  input_path: &Path
) -> String {
  let mut lines = Vec::new();
  lines.push(
    "# Hyperfine Summary".to_string()
  );
  lines.push(String::new());
  lines.push(format!(
    "source: `{}`",
    input_path.display()
  ));
  lines.push(format!(
    "commands: {}",
    report.results.len()
  ));
  lines.push(String::new());

  if report.results.is_empty() {
    lines.push(
      "No benchmark results found in \
       the input report."
        .to_string()
    );
    return lines.join("\n");
  }

  let mut rows = report.results.clone();
  rows.sort_by(|left, right| {
    left
      .mean
      .partial_cmp(&right.mean)
      .unwrap_or(
        std::cmp::Ordering::Equal
      )
  });
  let fastest =
    rows[0].mean.max(f64::EPSILON);

  lines
    .push("## Raw Results".to_string());
  lines.push(String::new());
  lines.push(
    "| Command | Mean (ms) | Stddev \
     (ms) | Ratio vs Fastest |"
      .to_string()
  );
  lines.push(
    "|---|---:|---:|---:|".to_string()
  );
  for row in &rows {
    let mean_ms = row.mean * 1000.0;
    let stddev_ms = row.stddev * 1000.0;
    let ratio = row.mean / fastest;
    lines.push(format!(
      "| `{}` | {:.3} | {:.3} | \
       {:.2}x |",
      row.command,
      mean_ms,
      stddev_ms,
      ratio
    ));
  }

  let mut grouped =
    BTreeMap::<String, SpeedPair>::new(
    );
  for row in &rows {
    if let Some((impl_name, task)) =
      parse_impl_and_task(&row.command)
    {
      let entry = grouped
        .entry(task.to_string())
        .or_default();
      match impl_name {
        | "ruby" => {
          entry.ruby = Some(row.mean);
        }
        | "rust" => {
          entry.rust = Some(row.mean);
        }
        | _ => {}
      }
    }
  }

  let parity_rows = grouped
    .iter()
    .filter_map(|(task, pair)| {
      let ruby = pair.ruby?;
      let rust = pair.rust?;
      let speedup =
        ruby / rust.max(f64::EPSILON);
      Some((
        task.clone(),
        ruby,
        rust,
        speedup
      ))
    })
    .collect::<Vec<_>>();
  if !parity_rows.is_empty() {
    lines.push(String::new());
    lines.push(
      "## Ruby vs Rust".to_string()
    );
    lines.push(String::new());
    lines.push(
      "| Task | Ruby Mean (ms) | Rust \
       Mean (ms) | Rust Speedup |"
        .to_string()
    );
    lines.push(
      "|---|---:|---:|---:|"
        .to_string()
    );
    for (task, ruby, rust, speedup) in
      parity_rows
    {
      lines.push(format!(
        "| `{task}` | {:.3} | {:.3} | \
         {:.2}x |",
        ruby * 1000.0,
        rust * 1000.0,
        speedup
      ));
    }
  }

  lines.join("\n")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default_output_path_uses_stem() {
    let input = Path::new(
      "target/reports/\
       benchmark-ruby-parity.json"
    );
    let output =
      default_output_path(input);
    assert_eq!(
      output,
      PathBuf::from(
        "target/reports/\
         benchmark-ruby-parity-summary.\
         md"
      )
    );
  }

  #[test]
  fn render_report_includes_parity_table()
   {
    let report: HyperfineReport = serde_json::from_str(
      r#"{
        "results": [
          {"command":"ruby:parse-json","mean":2.0,"stddev":0.1},
          {"command":"rust:parse-json","mean":1.0,"stddev":0.05},
          {"command":"rust:sample-json","mean":0.2,"stddev":0.01}
        ]
      }"#,
    )
    .expect("parse hyperfine sample");
    let markdown = render_report(
      &report,
      Path::new(
        "target/reports/in.json"
      )
    );
    assert!(
      markdown
        .contains("## Raw Results")
    );
    assert!(
      markdown
        .contains("## Ruby vs Rust")
    );
    assert!(
      markdown.contains("`parse-json`")
    );
    assert!(markdown.contains("2.00x"));
  }
}
