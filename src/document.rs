use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct Document {
  pages: Vec<Page>
}

#[derive(Debug, Clone)]
pub struct Page {
  text: String
}

impl Page {
  pub fn text(&self) -> &str {
    &self.text
  }
}

impl Document {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn from_text(text: &str) -> Self {
    let pages = text
      .split('\u{000C}')
      .map(|segment| {
        Page {
          text: segment
            .trim()
            .to_string()
        }
      })
      .collect::<Vec<_>>();

    if pages.is_empty() {
      Self::default()
    } else {
      Self {
        pages
      }
    }
  }

  pub fn open<P: AsRef<Path>>(
    path: P
  ) -> Self {
    let data = fs::read_to_string(path)
      .unwrap_or_default();
    Self::from_text(&data)
  }

  pub fn pages(&self) -> Vec<Page> {
    self.pages.clone()
  }

  pub fn add_page(
    &mut self,
    page: Page
  ) {
    self.pages.push(page);
  }
}
