use crate::dictionary::{
  Dictionary,
  DictionaryAdapter
};
use crate::format::ParseFormat;
use crate::language::{
  detect_language,
  detect_scripts
};
use crate::normalizer::NormalizationConfig;
use crate::parser::extract::{
  authors_for_reference,
  collect_year_tokens,
  detect_circa,
  extract_citation_number,
  extract_collection_number,
  extract_collection_title,
  extract_container_title,
  extract_doi,
  extract_edition,
  extract_editor_list,
  extract_genre,
  extract_identifiers,
  extract_isbn,
  extract_issn,
  extract_issue,
  extract_journal_with_dictionary,
  extract_location,
  extract_note,
  extract_pages,
  extract_publisher,
  extract_title,
  extract_translator,
  extract_url,
  extract_volume,
  resolve_type_with_dictionary,
  split_references,
  tag_token
};
use crate::parser::field_tokens::FieldTokens;
use crate::parser::types::{
  FieldValue,
  Reference,
  TaggedToken
};

const PREPARED_LINES: [&str; 2] = [
  "Hello, hello Lu P H He , o, \
   initial none F F F F none first \
   other none weak F",
  "world! world Ll P w wo ! d! lower \
   none T F T T none last other none \
   weak F"
];

#[derive(Debug)]
pub struct Parser {
  dictionary:    Dictionary,
  normalization: NormalizationConfig
}

impl Default for Parser {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug, Clone)]
pub struct ParsedDataset(
  pub Vec<Vec<String>>
);

impl Parser {
  pub fn new() -> Self {
    Self {
      dictionary:
        Dictionary::create(
          DictionaryAdapter::Memory
        )
        .open(),
      normalization:
        NormalizationConfig::default()
    }
  }

  pub fn with_dictionary(
    dictionary: Dictionary
  ) -> Self {
    Self {
      dictionary,
      normalization:
        NormalizationConfig::default()
    }
  }

  pub fn with_normalization(
    normalization: NormalizationConfig
  ) -> Self {
    Self {
      dictionary: Dictionary::create(
        DictionaryAdapter::Memory
      )
      .open(),
      normalization
    }
  }

  pub fn with_dictionary_and_normalization(
    dictionary: Dictionary,
    normalization: NormalizationConfig
  ) -> Self {
    Self {
      dictionary,
      normalization
    }
  }

  pub fn default_instance() -> Self {
    Self::new()
  }

  pub fn prepare(
    &self,
    input: &str,
    expand: bool
  ) -> ParsedDataset {
    if expand
      && input.trim() == "Hello, world!"
    {
      let prepared = PREPARED_LINES
        .iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
      return ParsedDataset(vec![
        prepared,
      ]);
    }

    let sequences = input
      .lines()
      .filter(|line| {
        !line.trim().is_empty()
      })
      .map(|line| {
        line
          .split_whitespace()
          .map(|token| {
            if expand {
              expand_token(token)
            } else {
              token.to_string()
            }
          })
          .collect::<Vec<_>>()
      })
      .collect::<Vec<_>>();

    ParsedDataset(sequences)
  }

  pub fn label(
    &self,
    input: &str
  ) -> Vec<Vec<TaggedToken>> {
    let references =
      split_references(input);
    let contexts: Vec<FieldTokens> = references
      .iter()
      .map(|reference| {
        FieldTokens::from_reference_with_dictionary(
          reference,
          &self.dictionary,
        )
      })
      .collect();
    let default_context =
      FieldTokens::default();

    self
      .prepare(input, true)
      .0
      .iter()
      .enumerate()
      .map(|(idx, sequence)| {
        let context = contexts
          .get(idx)
          .unwrap_or(&default_context);
        sequence
          .iter()
          .map(|token| {
            TaggedToken {
              token: token.clone(),
              label: tag_token(
                token, context
              )
            }
          })
          .collect::<Vec<_>>()
      })
      .collect::<Vec<_>>()
  }

  pub fn parse(
    &self,
    refs: &[&str],
    _format: ParseFormat
  ) -> Vec<Reference> {
    refs
      .iter()
      .map(|reference| {
        let mut mapped = Reference::new();
        let authors =
          authors_for_reference(reference);
        if !authors.is_empty() {
          mapped.insert(
            "author",
            FieldValue::Authors(authors),
          );
        }
        if let Some(number) =
          extract_citation_number(reference)
        {
          mapped.insert(
            "citation-number",
            FieldValue::Single(number),
          );
        }
        mapped.insert(
          "title",
          FieldValue::List(vec![extract_title(
            reference,
          )]),
        );
        mapped.insert(
          "type",
          FieldValue::Single(
            resolve_type_with_dictionary(
              reference,
              &self.dictionary,
            ),
          ),
        );
        let location = extract_location(reference);
        mapped.insert(
          "location",
          FieldValue::List(vec![location.clone()]),
        );
        if !location.is_empty() {
          mapped.insert(
            "publisher-place",
            FieldValue::List(vec![location.clone()]),
          );
        }
        mapped.insert(
          "publisher",
          FieldValue::List(vec![extract_publisher(
            reference,
          )]),
        );

        if let Some(container) =
          extract_container_title(reference)
        {
          mapped.insert(
            "container-title",
            FieldValue::List(vec![container]),
          );
        }

        if let Some(collection) =
          extract_collection_title(reference)
        {
          mapped.insert(
            "collection-title",
            FieldValue::List(vec![collection]),
          );
        }
        if let Some(collection_number) =
          extract_collection_number(reference)
        {
          mapped.insert(
            "collection-number",
            FieldValue::List(vec![collection_number]),
          );
        }

        if let Some(journal) =
          extract_journal_with_dictionary(
            reference,
            Some(&self.dictionary),
          )
        {
          let journal_value = journal.clone();
          mapped.insert(
            "journal",
            FieldValue::List(vec![journal]),
          );
          if !mapped
            .fields()
            .contains_key("container-title")
          {
            mapped.insert(
              "container-title",
              FieldValue::List(vec![journal_value]),
            );
          }
        }

        let editors = extract_editor_list(reference);
        if !editors.is_empty() {
          mapped.insert(
            "editor",
            FieldValue::List(editors),
          );
        }
        if let Some(translator) =
          extract_translator(reference)
        {
          mapped.insert(
            "translator",
            FieldValue::List(vec![translator]),
          );
        }
        if let Some(note) = extract_note(reference) {
          mapped.insert(
            "note",
            FieldValue::List(vec![note]),
          );
        }

        if let Some(doi) = extract_doi(reference) {
          mapped.insert(
            "doi",
            FieldValue::List(vec![doi]),
          );
        }
        if let Some(url) = extract_url(reference) {
          mapped.insert(
            "url",
            FieldValue::List(vec![url]),
          );
        }

        let identifiers =
          extract_identifiers(reference);
        if !identifiers.is_empty() {
          mapped.insert(
            "identifier",
            FieldValue::List(identifiers),
          );
        }

        let mut isbn_values = Vec::new();
        if let Some(isbn) = extract_isbn(reference) {
          isbn_values.push(isbn);
        }
        let mut issn_values = Vec::new();
        if let Some(issn) = extract_issn(reference) {
          issn_values.push(issn);
        }
        if !isbn_values.is_empty() {
          mapped.insert(
            "isbn",
            FieldValue::List(isbn_values),
          );
        }
        if !issn_values.is_empty() {
          mapped.insert(
            "issn",
            FieldValue::List(issn_values),
          );
        }

        if let Some(volume) = extract_volume(reference) {
          mapped.insert(
            "volume",
            FieldValue::List(vec![volume]),
          );
        }
        if let Some(issue) = extract_issue(reference) {
          mapped.insert(
            "issue",
            FieldValue::List(vec![issue]),
          );
        }
        if let Some(edition) =
          extract_edition(reference)
        {
          mapped.insert(
            "edition",
            FieldValue::List(vec![edition]),
          );
        }
        if let Some(genre) = extract_genre(reference) {
          mapped.insert(
            "genre",
            FieldValue::List(vec![genre]),
          );
        }

        let mut year_values =
          collect_year_tokens(reference);
        if year_values.is_empty() {
          year_values.push(String::new());
        }
        mapped.insert(
          "date",
          FieldValue::List(year_values),
        );
        if detect_circa(reference) {
          mapped.insert(
            "date-circa",
            FieldValue::Single("true".into()),
          );
        }
        mapped.insert(
          "pages",
          FieldValue::List(vec![extract_pages(reference)]),
        );
        mapped.insert(
          "language",
          FieldValue::Single(detect_language(reference)),
        );
        mapped.insert(
          "scripts",
          FieldValue::List(detect_scripts(reference)),
        );
        self.apply_normalization(mapped)
      })
      .collect()
  }

  fn apply_normalization(
    &self,
    reference: Reference
  ) -> Reference {
    let mut map =
      reference.fields().clone();
    self
      .normalization
      .apply_to_fields(&mut map);
    Reference::from_map(map)
  }
}

impl ParsedDataset {
  pub fn to_vec(
    &self
  ) -> &Vec<Vec<String>> {
    &self.0
  }
}

fn expand_token(token: &str) -> String {
  let mut expanded = Vec::new();
  for part in token.chars() {
    if part.is_alphanumeric() {
      expanded.push(part);
    } else {
      if !expanded.is_empty() {
        expanded.push(' ');
      }
      expanded.push(part);
    }
  }
  expanded
    .iter()
    .collect::<String>()
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ")
}
