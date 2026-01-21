pub struct Document;

pub struct Page;

impl Document {
  pub fn new() -> Self {
    Self
  }

  pub fn open<
    P: AsRef<std::path::Path>
  >(
    _path: P
  ) -> Self {
    todo!(
      "Document opening is pending \
       implementation"
    )
  }

  pub fn pages(&self) -> Vec<Page> {
    todo!(
      "Document page extraction is \
       pending implementation"
    )
  }
}
