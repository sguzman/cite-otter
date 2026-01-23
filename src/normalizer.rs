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

pub mod location {
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
      input: &str
    ) -> (String, Option<String>) {
      let trimmed = input.trim();
      if let Some((before, after)) =
        trimmed.split_once(':')
      {
        (
          before.trim().to_string(),
          Some(
            after.trim().to_string()
          )
        )
      } else {
        (trimmed.to_string(), None)
      }
    }
  }
}

pub mod container {
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
      input: &str
    ) -> String {
      let mut value = input.trim();
      for prefix in &[
        "in ",
        "In ",
        "of ",
        "Presented at ",
        "presented at "
      ] {
        if value.starts_with(prefix) {
          value = value[prefix.len()..]
            .trim();
        }
      }
      value
        .trim_end_matches(|c: char| {
          c == ',' || c == '.'
        })
        .trim()
        .to_string()
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
