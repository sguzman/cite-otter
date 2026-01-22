use cite_otter::feature::number::{
  Feature,
  Observation
};

const YEAR_TOKENS: [&str; 5] = [
  "(1992)", "1992.", "2011,", "1776;",
  "1970/71"
];

#[test]
fn number_feature_detects_years() {
  let feature = Feature::new();

  for token in YEAR_TOKENS {
    assert_eq!(
      feature.observe(token),
      Observation::Year,
      "Number feature should treat \
       {token} as a year"
    );
  }
}
