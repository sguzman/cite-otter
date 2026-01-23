use crate::document::Document;
use crate::parser::{
  Parser,
  sequence_signature
};
use crate::sequence_model::SequenceModel;

#[derive(Debug, Clone)]
pub struct Finder {
  signatures: SequenceModel
}

impl Default for Finder {
  fn default() -> Self {
    Self::new()
  }
}

impl Finder {
  pub fn new() -> Self {
    Self {
      signatures:
        SequenceModel::default()
    }
  }

  pub fn with_signatures(
    signatures: SequenceModel
  ) -> Self {
    Self {
      signatures
    }
  }

  pub fn segments(
    input: &str
  ) -> Vec<String> {
    split_into_references(input)
  }

  pub fn label(
    &self,
    input: &str
  ) -> Vec<Document> {
    let segments =
      Self::segments(input);
    if segments.is_empty() {
      return vec![Document::from_text(
        input
      )];
    }

    let parser = Parser::new();
    let mut results = Vec::new();
    let has_signatures =
      self.signatures.has_signatures();

    for segment in segments {
      let sequences = parser
        .prepare(&segment, true)
        .0;
      let mut matched = false;

      for sequence in &sequences {
        let signature =
          sequence_signature(sequence);
        if has_signatures
          && self
            .signatures
            .should_match(&signature, 1)
        {
          matched = true;
          break;
        }
      }

      if matched || !has_signatures {
        results.push(
          Document::from_text(&segment)
        );
      }
    }

    if results.is_empty() {
      vec![Document::from_text(input)]
    } else {
      results
    }
  }
}

fn split_into_references(
  input: &str
) -> Vec<String> {
  input
    .split("\n\n")
    .map(str::trim)
    .filter(|seg| {
      !seg.is_empty()
        && seg
          .chars()
          .any(|c| c.is_ascii_digit())
        && seg.len() > 20
    })
    .map(|seg| seg.to_string())
    .collect()
}
