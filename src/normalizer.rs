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
      _input: &str,
      _prev: Option<&[&str]>
    ) -> Vec<String> {
      todo!(
        "Name normalization is \
         pending implementation"
      )
    }
  }
}
