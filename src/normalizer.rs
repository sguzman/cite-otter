pub mod names {
  #[derive(Debug, Clone)]
  pub struct Normalizer;

  impl Default for Normalizer {
    fn default() -> Self {
      Self::new()
    }
  }

  impl Normalizer {
    pub fn new() -> Self {
      Self
    }

    pub fn normalize(
      &self,
      input: &str,
      prev: Option<&[&str]>
    ) -> Vec<String> {
      let trimmed = input.trim();
      let cleaned = trimmed
        .trim_end_matches(',')
        .to_string();

      if let Some(first) = prev
        .and_then(|previous| {
          previous.first()
        })
        .filter(|_| {
          super::is_repeater(trimmed)
        })
      {
        return vec![first.to_string()];
      }

      vec![cleaned]
    }
  }
}

fn is_repeater(value: &str) -> bool {
  let allowed = ['-', '.', ',', ' '];
  !value.is_empty()
    && value.chars().all(|c| {
      c.is_whitespace()
        || allowed.contains(&c)
    })
}
