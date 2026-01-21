pub mod names {
  #[derive(Debug, Clone)]
  pub struct Normalizer;

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
