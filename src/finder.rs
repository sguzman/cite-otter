use crate::document::Document;
use crate::model::FinderModel;
use crate::parser::Parser;

#[derive(Debug, Clone)]
pub struct Finder {
  model: FinderModel
}

impl Default for Finder {
  fn default() -> Self {
    Self::new()
  }
}

impl Finder {
  pub fn new() -> Self {
    Self {
      model: FinderModel::default()
    }
  }

  pub fn with_model(
    model: FinderModel
  ) -> Self {
    Self {
      model
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
    let has_signatures =
      self.model.has_signatures();
    let mut results = Vec::new();

    for segment in segments {
      let sequences = parser
        .prepare(&segment, true)
        .0;
      let mut matched = false;

      for sequence in &sequences {
        let signature =
          token_sequence_signature(
            sequence
          );
        if has_signatures
          && self
            .model
            .contains_signature(
              &signature
            )
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
    .collect::<Vec<_>>()
}

fn token_sequence_signature(
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
