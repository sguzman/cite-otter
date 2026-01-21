pub mod number {
  #[derive(
    Debug, Clone, Copy, PartialEq, Eq,
  )]
  pub enum Observation {
    Year
  }

  #[derive(Debug)]
  pub struct Feature;

  impl Feature {
    pub fn new() -> Self {
      Self
    }

    pub fn observe(
      &self,
      _token: &str
    ) -> Observation {
      todo!(
        "Number feature observation \
         is pending implementation"
      )
    }
  }
}
