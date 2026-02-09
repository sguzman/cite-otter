use std::collections::BTreeSet;

use super::extract::{
  extract_collection_title,
  extract_container_title,
  extract_edition,
  extract_editor,
  extract_genre,
  extract_issue,
  extract_journal,
  extract_location,
  extract_note,
  extract_pages,
  extract_publisher,
  extract_title,
  extract_translator,
  extract_volume,
  normalize_token,
  tokens_from_authors,
  tokens_from_dates,
  tokens_from_identifiers,
  tokens_from_segment
};
use crate::dictionary::{
  Dictionary,
  DictionaryCode
};

#[derive(Debug, Clone, Default)]
pub(super) struct FieldTokens {
  pub(super) author: BTreeSet<String>,
  pub(super) title: BTreeSet<String>,
  pub(super) location: BTreeSet<String>,
  pub(super) publisher:
    BTreeSet<String>,
  pub(super) date: BTreeSet<String>,
  pub(super) pages: BTreeSet<String>,
  pub(super) container:
    BTreeSet<String>,
  pub(super) collection:
    BTreeSet<String>,
  pub(super) journal: BTreeSet<String>,
  pub(super) editor: BTreeSet<String>,
  pub(super) translator:
    BTreeSet<String>,
  pub(super) note: BTreeSet<String>,
  pub(super) identifier:
    BTreeSet<String>,
  pub(super) volume: BTreeSet<String>,
  pub(super) issue: BTreeSet<String>,
  pub(super) genre: BTreeSet<String>,
  pub(super) edition: BTreeSet<String>
}

impl FieldTokens {
  pub(super) fn from_reference(
    reference: &str
  ) -> Self {
    Self {
      author:     tokens_from_authors(
        reference
      ),
      title:      tokens_from_segment(
        &extract_title(reference)
      ),
      location:   tokens_from_segment(
        &extract_location(reference)
      ),
      publisher:  tokens_from_segment(
        &extract_publisher(reference)
      ),
      date:       tokens_from_dates(
        reference
      ),
      pages:      tokens_from_segment(
        &extract_pages(reference)
      ),
      container:  tokens_from_segment(
        extract_container_title(
          reference
        )
        .unwrap_or_default()
        .as_str()
      ),
      collection: tokens_from_segment(
        extract_collection_title(
          reference
        )
        .unwrap_or_default()
        .as_str()
      ),
      journal:    tokens_from_segment(
        extract_journal(reference)
          .unwrap_or_default()
          .as_str()
      ),
      editor:     tokens_from_segment(
        extract_editor(reference)
          .unwrap_or_default()
          .as_str()
      ),
      translator: tokens_from_segment(
        extract_translator(reference)
          .unwrap_or_default()
          .as_str()
      ),
      note:       tokens_from_segment(
        extract_note(reference)
          .unwrap_or_default()
          .as_str()
      ),
      identifier:
        tokens_from_identifiers(
          reference
        ),
      volume:     tokens_from_segment(
        extract_volume(reference)
          .unwrap_or_default()
          .as_str()
      ),
      issue:      tokens_from_segment(
        extract_issue(reference)
          .unwrap_or_default()
          .as_str()
      ),
      genre:      tokens_from_segment(
        extract_genre(reference)
          .unwrap_or_default()
          .as_str()
      ),
      edition:    tokens_from_segment(
        extract_edition(reference)
          .unwrap_or_default()
          .as_str()
      )
    }
  }

  pub(super) fn from_reference_with_dictionary(
    reference: &str,
    dictionary: &Dictionary
  ) -> Self {
    let mut tokens =
      Self::from_reference(reference);
    tokens.apply_dictionary(
      reference, dictionary
    );
    tokens
  }

  pub(super) fn apply_dictionary(
    &mut self,
    reference: &str,
    dictionary: &Dictionary
  ) {
    for term in
      reference.split(|c: char| {
        !c.is_alphanumeric()
      })
    {
      let term = term.trim();
      if term.is_empty() {
        continue;
      }
      let normalized =
        normalize_token(term);
      if normalized.is_empty() {
        continue;
      }
      for code in
        dictionary.lookup(term)
      {
        match code {
          | DictionaryCode::Name => {
            self.author
              .insert(normalized.clone());
          }
          | DictionaryCode::Place => {
            self.location
              .insert(normalized.clone());
          }
          | DictionaryCode::Publisher => {
            self.publisher
              .insert(normalized.clone());
          }
          | DictionaryCode::Journal => {
            self.journal
              .insert(normalized.clone());
          }
        }
      }
    }
  }
}
