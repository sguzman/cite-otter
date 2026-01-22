pub mod number {
  #[derive(
    Debug, Clone, Copy, PartialEq, Eq,
  )]
  pub enum Observation {
    Year,
    Unknown
  }

  #[derive(Debug)]
  pub struct Feature;

  impl Default for Feature {
    fn default() -> Self {
      Self
    }
  }

  impl Feature {
    pub fn new() -> Self {
      Self
    }

    pub fn observe(
      &self,
      token: &str
    ) -> Observation {
      if token
        .chars()
        .filter(|c| c.is_ascii_digit())
        .count()
        >= 4
      {
        return Observation::Year;
      }

      let digits = token
        .chars()
        .filter(|c| c.is_ascii_digit())
        .count();

      if digits >= 3 {
        Observation::Year
      } else {
        Observation::Unknown
      }
    }
  }
}
