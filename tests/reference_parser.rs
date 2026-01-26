use std::collections::{
  BTreeMap,
  BTreeSet
};

use cite_otter::dictionary::{
  Dictionary,
  DictionaryAdapter,
  DictionaryCode
};
use cite_otter::format::ParseFormat;
use std::fs;
use cite_otter::normalizer::{
  abbreviations::AbbreviationMap,
  NormalizationConfig
};
use cite_otter::parser::{
  Author,
  FieldValue,
  Parser
};

const PREPARED_LINES: [&str; 2] = [
  "Hello, hello Lu P H He , o, \
   initial none F F F F none first \
   other none weak F",
  "world! world Ll P w wo ! d! lower \
   none T F T T none last other none \
   weak F"
];

const PEREC_REF: &str =
  "Perec, Georges. A Void. London: \
   The Harvill Press, 1995. p.108.";

const PEREC_REF_NO_COMMA: &str =
  "Georges Perec. A Void. London: The \
   Harvill Press, 1995. p.108.";

const PEREC_MULTI_YEAR_REF: &str =
  "Perec, Georges. A Void. London: \
   The Harvill Press, 1995/96. p.108.";
const DATE_RANGE_REF: &str =
  "Perec, Georges. A Void. London: \
   The Harvill Press, 1995-04-02. pp. \
   12-34.";
const MONTH_NAME_REF: &str =
  "Perec, Georges. A Void. London: \
   The Harvill Press, Apr 5 1995. pp. \
   12-34.";
const YEAR_RANGE_REF: &str =
  "Perec, Georges. A Void. London: \
   The Harvill Press, 1995–1996.";
const MONTH_RANGE_REF: &str =
  "Perec, Georges. A Void. London: \
   The Harvill Press, Apr–May 1995.";
const MONTH_RANGE_DAY_REF: &str =
  "Perec, Georges. A Void. London: \
   The Harvill Press, Apr 5–7 1995.";
const MONTH_RANGE_DAY_PUNCT_REF: &str =
  "Perec, Georges. A Void. London: The \
   Harvill Press, Apr. 5–7, 1995.";
const DAY_RANGE_BEFORE_MONTH_REF: &str =
  "Perec, Georges. A Void. London: The \
   Harvill Press, 5–7 Apr 1995.";
const PUNCTUATED_AUTHORS_REF: &str =
  "Doe, J.; Smith, A., Jr.; O'Neil, \
   M.-J. Title. City: Pub, 2020.";
const PUNCTUATED_AUTHORS_WITH_AMP_REF: &str =
  "Doe, J.; Smith, A., Jr.; O'Neil, \
   M.-J.; Brown, R. & White, T. \
   Title. City: Pub, 2020.";
const FULL_NAME_COMMA_AUTHORS_REF:
  &str = "Perec, Georges, Calvino, \
          Italo. Title. City: Pub, \
          1999.";
const ET_AL_REF: &str =
  "Doe, J., et al. Title. City: Pub, \
   2020.";
const MONTH_NAME_PUNCT_REF: &str =
  "Perec, Georges. A Void. London: \
   The Harvill Press, Apr. 5, 1995.";
const MONTH_RANGE_LONG_REF: &str =
  "Perec, Georges. A Void. London: \
   The Harvill Press, Sept. 12–14, \
   2010.";

const MULTI_AUTHOR_REF: &str =
  "Doe, J. and Smith, A. A Title. \
   City: Pub, 2020.";

const COMPLEX_REF: &str =
  "Smith, Alice. On heuristics for \
   mixing metadata. Lecture Notes in \
   Computer Science, 4050. Journal of \
   Testing. Edited by Doe, J. \
   (Note: Preprint release). \
   doi:10.1000/test https://example.org.";

const TRANSLATOR_REF: &str =
  "Roe, Jane. Title. Translated by \
   Doe, J. ISBN 978-1-2345-6789-0 \
   ISSN 1234-5678.";
const DERRIDA_REF: &str =
  "Derrida, J. (c.1967). L’écriture \
   et la différence (1 éd.). Paris: \
   Éditions du Seuil.";
const QUOTED_TITLE_REF: &str =
  "Doe, Jane. \"Quoted Title\". City: \
   Pub, 2020.";
const PARTICLE_NAME_REF: &str =
  "van der Waals, J. D. A Title. \
   City: Pub, 2020.";
const GENRE_REF: &str =
  "Doe, Jane. A Title. [PhD thesis]. \
   City: Pub, 2020.";
const HEIDEGGER_REF: &str =
  "Heidegger M., 1927, Être et temps, \
   Gallimard, Ed. 1986, Paris.";
const CITATION_NUMBER_REF: &str =
  "60. Differences in cyclic fatigue \
   resistance between ProTaper Next \
   and ProTaper Universal instruments \
   at different levels. \
   Pérez-Higueras JJ, Arias A, de la \
   Macorra JC, Peters OA. septembre \
   2014, J Endod, Vol. 9, pp. 1477-81.";
const PAGE_RANGE_DATE_REF: &str =
  "Solon, G. (1999). Chapter 29. \
   Intergenerational mobility in the \
   labor market. (Vol. 3, Part A, pp. \
   1761–1800). London: Elsevier.";
const CHAPTER_IN_REF: &str =
  "Solon, G. (1999). Chapter 29. \
   Intergenerational mobility in the \
   labor market. In O. C. Ashenfelter \
   and D. Card (Eds.), Handbook of \
   labor economics (Vol. 3, Part A, \
   pp. 1761–1800). London: Elsevier.";
const VOLUME_ISSUE_REF: &str =
  "Ghezzi, C., Mandriolli, D., \
   Morzenti, A. Trio: A logic \
   language for executable \
   specifications of real-time \
   systems. Journal of Systems and \
   Software, 12(2), 107-123, May 1990.";
const ISSUE_PART_REF: &str =
  "Fischer, H. Centre Pompidou. \
   Deutsche Bauzeitung, H. 134, Part \
   3, 2000, 20-21.";
const IEEE_PROC_REF: &str =
  "Ramadge, P., Wonham, W. The \
   Control of Discrete Event Systems. \
   Proceedings of the IEEE, 77(1), \
   81-98, 1989.";
const ROMERO_REF: &str =
  "Romero, C., Paunesku, D., & Dweck, \
   C. (2011). Brainology in the \
   classroom: An online growth \
   mindset intervention affects GPA, \
   conduct, and implicit theories. \
   Poster session presented at \
   Society for Research in Child \
   Development Biennial Meeting, \
   Montreal, Canada.";
const J_ENDOD_REF: &str =
  "60. Differences in cyclic fatigue \
   resistance between ProTaper Next \
   and ProTaper Universal instruments \
   at different levels. \
   Pérez-Higueras JJ, Arias A, de la \
   Macorra JC, Peters OA. septembre \
   2014, J Endod, Vol. 9, pp. 1477-81.";
const FISCHER_REF: &str =
  "Fischer, H. Centre Pompidou. \
   Deutsche Bauzeitung, H. 134, Part \
   3, 2000, 20-21.";
const BRISCHOUX_REF: &str =
  "Brischoux, F., Chakraborty, S., \
   Brierley, D.I., Ungless, M.A. \
   Phasic excitation of dopamine \
   neurons in ventral VTA by noxious \
   stimuli. Proc Natl Acad Sci U S A \
   106, 4894-4899, 2009.";
const HAN_REF: &str =
  "[13] S. E. Han, G. Chen. Nano Lett \
   2010, 10, 1012.";

#[test]
fn prepare_returns_expanded_dataset() {
  let parser = Parser::new();
  let dataset = parser
    .prepare("Hello, world!", true);

  let expected: Vec<Vec<String>> = vec![
    PREPARED_LINES
      .iter()
      .map(|line| line.to_string())
      .collect(),
  ];

  assert_eq!(
    dataset.to_vec(),
    &expected,
    "parser.prepare should expand \
     tokens exactly as AnyStyle 1.x"
  );
}

#[test]
fn parse_returns_metadata_map() {
  let parser = Parser::new();
  let references = parser.parse(
    &[PEREC_REF],
    ParseFormat::Json
  );

  assert_eq!(
    references.len(),
    1,
    "Should return one reference"
  );

  let mut expected_fields =
    BTreeMap::new();
  expected_fields.insert(
    "title".into(),
    FieldValue::List(vec![
      "A Void".into(),
    ])
  );
  expected_fields.insert(
    "location".into(),
    FieldValue::List(vec![
      "London".into(),
    ])
  );
  expected_fields.insert(
    "publisher".into(),
    FieldValue::List(vec![
      "The Harvill Press".into(),
    ])
  );
  expected_fields.insert(
    "date".into(),
    FieldValue::List(vec![
      "1995".into(),
    ])
  );
  expected_fields.insert(
    "pages".into(),
    FieldValue::List(vec![
      "108".into(),
    ])
  );
  expected_fields.insert(
    "type".into(),
    FieldValue::Single("book".into())
  );

  let reference = &references[0].0;
  assert!(
    expected_fields.keys().all(
      |key: &String| {
        reference.contains_key(key)
      }
    ),
    "Expected parser.parse to \
     populate the documented fields"
  );
}

#[test]
fn parse_captures_collection_journal_editor_and_identifiers()
 {
  let parser = Parser::new();
  let references = parser.parse(
    &[COMPLEX_REF],
    ParseFormat::Json
  );

  let reference = &references[0].0;
  assert_list_field(
    reference,
    "collection-title",
    "Lecture Notes in Computer Science"
  );
  assert_list_field(
    reference,
    "collection-number",
    "4050"
  );
  assert_list_field(
    reference,
    "journal",
    "Journal of Testing"
  );
  assert_list_field(
    reference,
    "editor",
    "Edited by Doe"
  );
  assert_list_field(
    reference,
    "note",
    "Note: Preprint release"
  );
  assert_list_field(
    reference,
    "doi",
    "doi:10.1000/test"
  );
  assert_list_field(
    reference,
    "url",
    "https://example.org"
  );
}

#[test]
fn parse_builds_structured_authors_for_variant_formats()
 {
  let parser = Parser::new();
  let references = parser.parse(
    &[PEREC_REF, PEREC_REF_NO_COMMA],
    ParseFormat::Json
  );

  let expected = Author {
    family: "Perec".into(),
    given:  "Georges".into()
  };

  for reference in references {
    let author_field = reference
      .fields()
      .get("author")
      .expect(
        "parser should always emit an \
         author"
      );

    let authors = match author_field {
      FieldValue::Authors(list) => list,
      other => panic!(
        "Expected FieldValue::Authors, got {other:?}"
      )
    };

    assert!(
      authors.first()
        == Some(&expected),
      "Each reference should \
       normalize author components \
       consistently"
    );
  }
}

#[test]
fn parse_extracts_title_from_author_date_segment()
 {
  let parser = Parser::new();
  let references = parser.parse(
    &[HEIDEGGER_REF],
    ParseFormat::Json
  );
  let reference = &references[0].0;
  assert_list_field(
    reference,
    "title",
    "Être et temps"
  );
}

#[test]
fn parse_handles_multiple_authors_with_initials()
 {
  let parser = Parser::new();
  let references = parser.parse(
    &[MULTI_AUTHOR_REF],
    ParseFormat::Json
  );

  let author_field = references[0]
    .fields()
    .get("author")
    .expect("author field");
  let authors = match author_field {
    | FieldValue::Authors(list) => list,
    | other => {
      panic!(
      "Expected FieldValue::Authors, \
       got {other:?}"
    )
    }
  };
  assert_eq!(authors.len(), 2);
  assert_eq!(authors[0], Author {
    family: "Doe".into(),
    given:  "J".into()
  });
  assert_eq!(authors[1], Author {
    family: "Smith".into(),
    given:  "A".into()
  });
}

#[test]
fn parse_captures_translator_and_identifiers()
 {
  let parser = Parser::new();
  let references = parser.parse(
    &[TRANSLATOR_REF],
    ParseFormat::Json
  );

  let reference = &references[0].0;
  assert_list_field(
    reference,
    "translator",
    "Translated by Doe"
  );
  assert_list_field(
    reference,
    "isbn",
    "978-1-2345-6789-0"
  );
  assert_list_field(
    reference,
    "issn",
    "1234-5678"
  );
}

#[test]
fn parse_extracts_edition_from_parenthetical()
 {
  let parser = Parser::new();
  let references = parser.parse(
    &[DERRIDA_REF],
    ParseFormat::Json
  );

  let reference = &references[0].0;
  assert_list_field(
    reference, "edition", "1"
  );
  assert_list_field(
    reference,
    "title",
    "L’écriture et la différence"
  );
  let circa = reference
    .get("date-circa")
    .and_then(|value| {
      match value {
        | FieldValue::Single(text) => {
          Some(text.as_str())
        }
        | _ => None
      }
    });
  assert_eq!(
    circa,
    Some("true"),
    "parser should flag circa dates"
  );
}

#[test]
fn parse_strips_wrapping_quotes_from_titles()
 {
  let parser = Parser::new();
  let references = parser.parse(
    &[QUOTED_TITLE_REF],
    ParseFormat::Json
  );

  let reference = &references[0].0;
  assert_list_field(
    reference,
    "title",
    "Quoted Title"
  );
}

#[test]
fn parse_preserves_author_particles() {
  let parser = Parser::new();
  let references = parser.parse(
    &[PARTICLE_NAME_REF],
    ParseFormat::Json
  );

  let author_field = references[0]
    .fields()
    .get("author")
    .expect("author field");
  let authors = match author_field {
    | FieldValue::Authors(list) => list,
    | other => {
      panic!(
      "Expected FieldValue::Authors, \
       got {other:?}"
    )
    }
  };
  assert_eq!(authors[0], Author {
    family: "van der Waals".into(),
    given:  "J D".into()
  });
}

#[test]
fn parse_extracts_genre_from_brackets()
{
  let parser = Parser::new();
  let references = parser.parse(
    &[GENRE_REF],
    ParseFormat::Json
  );

  let reference = &references[0].0;
  assert_list_field(
    reference,
    "genre",
    "PhD thesis"
  );
}

#[test]
fn parse_uses_dictionary_for_type_resolution()
 {
  let mut dictionary =
    Dictionary::create(
      DictionaryAdapter::Memory
    )
    .open();
  dictionary
    .import_entries(vec![(
      "Nature".to_string(),
      DictionaryCode::Journal.bit()
    )])
    .expect("dictionary import");
  let parser =
    Parser::with_dictionary(dictionary);

  let references = parser.parse(
    &["Doe, J. Nature. 2020."],
    ParseFormat::Json
  );
  let reference = &references[0].0;
  let parsed = reference
    .get("type")
    .expect("type field");
  match parsed {
    | FieldValue::Single(value) => {
      assert_eq!(
        value, "article",
        "dictionary journal tag \
         should set article type"
      );
    }
    | _ => {
      panic!(
        "expected single type value"
      )
    }
  }
}

#[test]
fn parse_applies_normalization_to_publisher()
 {
  let abbreviations =
    AbbreviationMap::load_from_str(
      "Univ. Press\tUniversity Press"
    );
  let config =
    NormalizationConfig::default()
      .with_publisher_abbrev(
        abbreviations
      );
  let parser =
    Parser::with_normalization(config);

  let references = parser.parse(
    &["Doe, J. Title. City: Univ. \
       Press, 2020."],
    ParseFormat::Json
  );
  let reference = &references[0].0;
  let publisher = reference
    .get("publisher")
    .and_then(|value| {
      match value {
        | FieldValue::List(items) => {
          items.first().cloned()
        }
        | FieldValue::Single(text) => {
          Some(text.clone())
        }
        | _ => None
      }
    });
  assert_eq!(
    publisher.as_deref(),
    Some("University Press")
  );
}

#[test]
fn parse_applies_normalization_to_container()
 {
  let container_text =
    fs::read_to_string(
      "tests/fixtures/\
       container-abbrev-sample.txt"
    )
    .expect("container fixture");
  let config =
    NormalizationConfig::default()
      .with_container_abbrev(
        AbbreviationMap::load_from_str(
          &container_text
        )
      );
  let parser =
    Parser::with_normalization(config);

  let references = parser.parse(
    &["Doe, J. Proc. Test. \
       Proceedings of Testing. City: \
       Pub, 2020."],
    ParseFormat::Json
  );
  let reference = &references[0].0;
  let container = reference
    .get("container-title")
    .and_then(|value| {
      match value {
        | FieldValue::List(items) => {
          items.first().cloned()
        }
        | FieldValue::Single(text) => {
          Some(text.clone())
        }
        | _ => None
      }
    });
  assert_eq!(
    container.as_deref(),
    Some("Proceedings of Testing")
  );
}

#[test]
fn parse_uses_normalization_dir_assets()
{
  let config =
    NormalizationConfig::load_from_dir(
      std::path::Path::new(
        "tests/fixtures/\
         normalization-dir"
      )
    )
    .expect("load normalization dir");
  let parser =
    Parser::with_normalization(config);

  let references = parser.parse(
    &["Doe, J. Proc. Test. \
       Proceedings of Testing. City: \
       Univ. Press, 2020."],
    ParseFormat::Json
  );
  let reference = &references[0].0;
  let publisher = reference
    .get("publisher")
    .and_then(|value| {
      match value {
        | FieldValue::List(items) => {
          items.first().cloned()
        }
        | FieldValue::Single(text) => {
          Some(text.clone())
        }
        | _ => None
      }
    });
  assert_eq!(
    publisher.as_deref(),
    Some("University Press")
  );
  let language = reference
    .get("language")
    .and_then(|value| {
      match value {
        | FieldValue::Single(text) => {
          Some(text.as_str())
        }
        | _ => None
      }
    });
  assert_eq!(
    language,
    Some("en-US"),
    "locale overrides should update \
     language"
  );
  let scripts = reference
    .get("scripts")
    .and_then(|value| {
      match value {
        | FieldValue::List(items) => {
          Some(items.clone())
        }
        | FieldValue::Single(text) => {
          Some(vec![text.clone()])
        }
        | _ => None
      }
    })
    .unwrap_or_default();
  assert!(
    scripts
      .iter()
      .any(|value| { value == "Latn" }),
    "locale overrides should update \
     script names"
  );
  let container_values = reference
    .get("container-title")
    .and_then(|value| {
      match value {
        | FieldValue::List(items) => {
          Some(items.clone())
        }
        | FieldValue::Single(text) => {
          Some(vec![text.clone()])
        }
        | _ => None
      }
    })
    .unwrap_or_default();
  assert!(container_values.iter().any(
    |value| {
      value == "Proceedings of Testing"
    }
  ));
}

#[test]
fn parse_prefers_first_year_in_multi_year_tokens()
 {
  let parser = Parser::new();
  let references = parser.parse(
    &[PEREC_MULTI_YEAR_REF],
    ParseFormat::Json
  );

  let date_values = match references[0]
    .fields()
    .get("date")
  {
    | Some(FieldValue::List(
      values
    )) => values,
    | other => {
      panic!(
        "Expected list of date \
         tokens, got {other:?}"
      )
    }
  };

  let expected: Vec<String> =
    vec!["1995".into(), "1996".into()];
  assert_eq!(
    date_values, &expected,
    "Parser should normalize the \
     multi-year range"
  );
}

#[test]
fn parse_captures_page_ranges_and_date_parts()
 {
  let parser = Parser::new();
  let references = parser.parse(
    &[DATE_RANGE_REF],
    ParseFormat::Json
  );

  let reference = &references[0].0;
  assert_list_field(
    reference, "pages", "12-34"
  );

  let date_values =
    match reference.get("date") {
      | Some(FieldValue::List(
        values
      )) => values,
      | other => {
        panic!(
          "Expected list of date \
           values, got {other:?}"
        )
      }
    };
  assert_eq!(
    date_values,
    &vec![
      "1995".to_string(),
      "04".to_string(),
      "02".to_string()
    ],
    "Parser should capture date parts"
  );
}

#[test]
fn parse_captures_month_name_dates() {
  let parser = Parser::new();
  let references = parser.parse(
    &[MONTH_NAME_REF],
    ParseFormat::Json
  );

  let reference = &references[0].0;
  let date_values =
    match reference.get("date") {
      | Some(FieldValue::List(
        values
      )) => values,
      | other => {
        panic!(
          "Expected list of date \
           values, got {other:?}"
        )
      }
    };
  assert_eq!(
    date_values,
    &vec![
      "1995".to_string(),
      "04".to_string(),
      "5".to_string()
    ],
    "Parser should parse month names \
     into date parts"
  );
}

#[test]
fn parse_captures_year_ranges_with_en_dash()
 {
  let parser = Parser::new();
  let references = parser.parse(
    &[YEAR_RANGE_REF],
    ParseFormat::Json
  );

  let date_values = match references[0]
    .fields()
    .get("date")
  {
    | Some(FieldValue::List(
      values
    )) => values,
    | other => {
      panic!(
        "Expected list of date \
         values, got {other:?}"
      )
    }
  };

  assert_eq!(
    date_values,
    &vec![
      "1995".to_string(),
      "1996".to_string()
    ],
    "Parser should normalize en dash \
     year ranges"
  );
}

#[test]
fn parse_prefers_first_month_in_ranges()
{
  let parser = Parser::new();
  let references = parser.parse(
    &[MONTH_RANGE_REF],
    ParseFormat::Json
  );

  let date_values = match references[0]
    .fields()
    .get("date")
  {
    | Some(FieldValue::List(
      values
    )) => values,
    | other => {
      panic!(
        "Expected list of date \
         values, got {other:?}"
      )
    }
  };

  assert_eq!(
    date_values,
    &vec![
      "1995".to_string(),
      "04".to_string()
    ],
    "Parser should prefer the first \
     month in ranges"
  );
}

#[test]
fn parse_prefers_first_day_in_month_ranges()
 {
  let parser = Parser::new();
  let references = parser.parse(
    &[MONTH_RANGE_DAY_REF],
    ParseFormat::Json
  );

  let date_values = match references[0]
    .fields()
    .get("date")
  {
    | Some(FieldValue::List(
      values
    )) => values,
    | other => {
      panic!(
        "Expected list of date \
         values, got {other:?}"
      )
    }
  };

  assert_eq!(
    date_values,
    &vec![
      "1995".to_string(),
      "04".to_string(),
      "5".to_string()
    ],
    "Parser should prefer the first \
     day in month ranges"
  );
}

#[test]
fn parse_handles_month_ranges_with_punctuation()
 {
  let parser = Parser::new();
  let references = parser.parse(
    &[MONTH_RANGE_DAY_PUNCT_REF],
    ParseFormat::Json
  );

  let date_values = match references[0]
    .fields()
    .get("date")
  {
    | Some(FieldValue::List(
      values
    )) => values,
    | other => {
      panic!(
        "Expected list of date \
         values, got {other:?}"
      )
    }
  };

  assert_eq!(
    date_values,
    &vec![
      "1995".to_string(),
      "04".to_string(),
      "5".to_string()
    ],
    "Parser should capture month \
     range days with punctuation"
  );
}

#[test]
fn parse_handles_day_ranges_before_month_name()
 {
  let parser = Parser::new();
  let references = parser.parse(
    &[DAY_RANGE_BEFORE_MONTH_REF],
    ParseFormat::Json
  );

  let date_values = match references[0]
    .fields()
    .get("date")
  {
    | Some(FieldValue::List(
      values
    )) => values,
    | other => {
      panic!(
        "Expected list of date \
         values, got {other:?}"
      )
    }
  };

  assert_eq!(
    date_values,
    &vec![
      "1995".to_string(),
      "04".to_string(),
      "5".to_string()
    ],
    "Parser should capture day ranges \
     before month names"
  );
}

#[test]
fn parse_handles_punctuated_author_lists()
 {
  let parser = Parser::new();
  let references = parser.parse(
    &[PUNCTUATED_AUTHORS_REF],
    ParseFormat::Json
  );

  let author_field = references[0]
    .fields()
    .get("author")
    .expect("author field");
  let authors = match author_field {
    | FieldValue::Authors(list) => list,
    | other => {
      panic!(
      "Expected FieldValue::Authors, \
       got {other:?}"
    )
    }
  };
  assert_eq!(
    authors.len(),
    3,
    "parser should split \
     punctuation-heavy author lists"
  );
}

#[test]
fn parse_handles_punctuated_author_lists_with_ampersands()
 {
  let parser = Parser::new();
  let references = parser.parse(
    &[PUNCTUATED_AUTHORS_WITH_AMP_REF],
    ParseFormat::Json
  );

  let author_field = references[0]
    .fields()
    .get("author")
    .expect("author field");
  let authors = match author_field {
    | FieldValue::Authors(list) => list,
    | other => {
      panic!(
      "Expected FieldValue::Authors, \
       got {other:?}"
    )
    }
  };
  assert_eq!(
    authors.len(),
    5,
    "parser should split ampersand \
     punctuation-heavy author lists"
  );
}

#[test]
fn parse_handles_comma_separated_full_names()
 {
  let parser = Parser::new();
  let references = parser.parse(
    &[FULL_NAME_COMMA_AUTHORS_REF],
    ParseFormat::Json
  );

  let author_field = references[0]
    .fields()
    .get("author")
    .expect("author field");
  let authors = match author_field {
    | FieldValue::Authors(list) => list,
    | other => {
      panic!(
      "Expected FieldValue::Authors, \
       got {other:?}"
    )
    }
  };
  assert_eq!(
    authors.len(),
    2,
    "parser should split comma \
     separated full-name authors"
  );
}

#[test]
fn parse_handles_citation_number_author_lists()
 {
  let parser = Parser::new();
  let references = parser.parse(
    &[CITATION_NUMBER_REF],
    ParseFormat::Json
  );

  let author_field = references[0]
    .fields()
    .get("author")
    .expect("author field");
  let authors = match author_field {
    | FieldValue::Authors(list) => list,
    | other => {
      panic!(
      "Expected FieldValue::Authors, \
       got {other:?}"
    )
    }
  };
  assert_eq!(
    authors.len(),
    4,
    "parser should split authors \
     after citation numbers"
  );
  assert_eq!(authors[0], Author {
    family: "Pérez-Higueras".into(),
    given:  "JJ".into()
  });
}

#[test]
fn parse_extracts_citation_numbers() {
  let parser = Parser::new();
  let references = parser.parse(
    &[CITATION_NUMBER_REF],
    ParseFormat::Json
  );

  let value = references[0]
    .fields()
    .get("citation-number")
    .expect("citation-number field");
  let number = match value {
    | FieldValue::Single(text) => text,
    | other => {
      panic!(
        "Expected FieldValue::Single, \
         got {other:?}"
      )
    }
  };
  assert_eq!(
    number, "60.",
    "parser should capture citation \
     numbers including punctuation"
  );
}

#[test]
fn parse_skips_et_al_in_author_lists() {
  let parser = Parser::new();
  let references = parser.parse(
    &[ET_AL_REF],
    ParseFormat::Json
  );

  let author_field = references[0]
    .fields()
    .get("author")
    .expect("author field");
  let authors = match author_field {
    | FieldValue::Authors(list) => list,
    | other => {
      panic!(
      "Expected FieldValue::Authors, \
       got {other:?}"
    )
    }
  };
  assert_eq!(
    authors.len(),
    1,
    "parser should ignore et al \
     placeholders"
  );
  assert_eq!(authors[0], Author {
    family: "Doe".into(),
    given:  "J".into()
  });
}

#[test]
fn parse_captures_month_names_with_punctuation()
 {
  let parser = Parser::new();
  let references = parser.parse(
    &[MONTH_NAME_PUNCT_REF],
    ParseFormat::Json
  );

  let reference = &references[0].0;
  let date_values =
    match reference.get("date") {
      | Some(FieldValue::List(
        values
      )) => values,
      | other => {
        panic!(
          "Expected list of date \
           values, got {other:?}"
        )
      }
    };
  assert_eq!(
    date_values,
    &vec![
      "1995".to_string(),
      "04".to_string(),
      "5".to_string()
    ],
    "Parser should capture month \
     names with punctuation"
  );
}

#[test]
fn parse_handles_month_ranges_with_days_and_punctuation()
 {
  let parser = Parser::new();
  let references = parser.parse(
    &[MONTH_RANGE_LONG_REF],
    ParseFormat::Json
  );

  let date_values = match references[0]
    .fields()
    .get("date")
  {
    | Some(FieldValue::List(
      values
    )) => values,
    | other => {
      panic!(
        "Expected list of date \
         values, got {other:?}"
      )
    }
  };

  assert_eq!(
    date_values,
    &vec![
      "2010".to_string(),
      "09".to_string(),
      "12".to_string()
    ],
    "Parser should capture month \
     ranges with punctuation and days"
  );
}

#[test]
fn parse_ignores_page_ranges_in_dates()
{
  let parser = Parser::new();
  let references = parser.parse(
    &[PAGE_RANGE_DATE_REF],
    ParseFormat::Json
  );
  let reference = &references[0].0;
  let date_values =
    match reference.get("date") {
      | Some(FieldValue::List(
        values
      )) => values,
      | other => {
        panic!(
          "Expected list of date \
           values, got {other:?}"
        )
      }
    };
  assert!(
    date_values
      .contains(&"1999".to_string()),
    "Parser should keep the year from \
     date segments"
  );
  assert!(
    !date_values
      .iter()
      .any(|value| value == "1761"),
    "Parser should ignore page ranges \
     in date tokens"
  );
}

#[test]
fn parse_captures_chapter_container_and_editors()
 {
  let parser = Parser::new();
  let references = parser.parse(
    &[CHAPTER_IN_REF],
    ParseFormat::Json
  );

  let reference = &references[0].0;
  assert_list_field(
    reference,
    "container-title",
    "Handbook of labor economics"
  );
  assert_list_field(
    reference,
    "editor",
    "O. C. Ashenfelter"
  );
  assert_list_field(
    reference, "editor", "D. Card"
  );
  match reference.get("type") {
    | Some(FieldValue::Single(
      value
    )) => {
      assert_eq!(
        value, "chapter",
        "chapter references should be \
         typed as chapters"
      );
    }
    | other => {
      panic!(
        "Expected chapter type, got \
         {other:?}"
      )
    }
  }
}

#[test]
fn parse_captures_volume_with_parts() {
  let parser = Parser::new();
  let references = parser.parse(
    &[PAGE_RANGE_DATE_REF],
    ParseFormat::Json
  );

  let reference = &references[0].0;
  assert_list_field(
    reference,
    "volume",
    "3, Part A"
  );
}

#[test]
fn parse_captures_volume_and_issue_pairs()
 {
  let parser = Parser::new();
  let references = parser.parse(
    &[VOLUME_ISSUE_REF],
    ParseFormat::Json
  );

  let reference = &references[0].0;
  assert_list_field(
    reference, "volume", "12"
  );
  assert_list_field(
    reference, "issue", "2"
  );
}

#[test]
fn parse_captures_issue_parts() {
  let parser = Parser::new();
  let references = parser.parse(
    &[ISSUE_PART_REF],
    ParseFormat::Json
  );

  let reference = &references[0].0;
  assert_list_field(
    reference,
    "issue",
    "134, Part 3"
  );
}

#[test]
fn parse_prefers_journal_type_for_proceedings()
 {
  let parser = Parser::new();
  let references = parser.parse(
    &[IEEE_PROC_REF],
    ParseFormat::Json
  );

  let reference = &references[0].0;
  assert_list_field(
    reference,
    "journal",
    "Proceedings of the IEEE"
  );
  assert_list_field(
    reference, "pages", "81-98"
  );
  match reference.get("type") {
    | Some(FieldValue::Single(
      value
    )) => {
      assert_eq!(
        value, "article-journal",
        "type should prefer journal \
         classification"
      );
    }
    | other => {
      panic!(
        "Expected FieldValue::Single, \
         got {other:?}"
      )
    }
  }
}

#[test]
fn parse_extracts_etre_et_temps_title()
{
  let parser = Parser::new();
  let references = parser.parse(
    &[HEIDEGGER_REF],
    ParseFormat::Json
  );

  let reference = &references[0].0;
  assert_list_field(
    reference,
    "title",
    "Être et temps"
  );
}

#[test]
fn parse_extracts_romero_authors_and_title()
 {
  let parser = Parser::new();
  let references = parser.parse(
    &[ROMERO_REF],
    ParseFormat::Json
  );

  let reference = &references[0].0;
  assert_list_field(
    reference,
    "title",
    "Brainology in the classroom: An \
     online growth mindset \
     intervention affects GPA, \
     conduct, and implicit theories"
  );
  let authors =
    match reference.get("author") {
      | Some(FieldValue::Authors(
        list
      )) => list,
      | other => {
        panic!(
        "Expected FieldValue::Authors, \
         got {other:?}"
      )
      }
    };
  assert_eq!(
    authors.len(),
    3,
    "should capture all authors"
  );
  assert_eq!(authors[0], Author {
    family: "Romero".into(),
    given:  "C".into()
  });
  assert_list_field(
    reference,
    "container-title",
    "Poster session presented at \
     Society for Research in Child \
     Development Biennial Meeting"
  );
  assert!(
    !reference
      .contains_key("date-circa"),
    "Romero reference should not be \
     circa"
  );
}

#[test]
fn parse_extracts_j_endod_reference() {
  let parser = Parser::new();
  let references = parser.parse(
    &[J_ENDOD_REF],
    ParseFormat::Json
  );

  let reference = &references[0].0;
  assert_list_field(
    reference,
    "title",
    "Differences in cyclic fatigue \
     resistance between ProTaper Next \
     and ProTaper Universal \
     instruments at different levels"
  );
  assert_list_field(
    reference,
    "container-title",
    "J Endod"
  );
  assert_list_field(
    reference, "volume", "9"
  );
  assert_list_field(
    reference, "pages", "1477-81"
  );
  match reference.get("citation-number")
  {
    | Some(FieldValue::Single(
      value
    )) => {
      assert_eq!(value, "60.");
    }
    | other => {
      panic!(
        "Expected FieldValue::Single, \
         got {other:?}"
      )
    }
  }
  match reference.get("type") {
    | Some(FieldValue::Single(
      value
    )) => {
      assert_eq!(
        value,
        "article-journal"
      );
    }
    | other => {
      panic!(
        "Expected FieldValue::Single, \
         got {other:?}"
      )
    }
  }
}

#[test]
fn parse_extracts_fischer_reference() {
  let parser = Parser::new();
  let references = parser.parse(
    &[FISCHER_REF],
    ParseFormat::Json
  );

  let reference = &references[0].0;
  assert_list_field(
    reference,
    "title",
    "Centre Pompidou"
  );
  assert_list_field(
    reference,
    "journal",
    "Deutsche Bauzeitung"
  );
  assert_list_field(
    reference,
    "issue",
    "134, Part 3"
  );
  assert_list_field(
    reference, "pages", "20-21"
  );
  match reference.get("type") {
    | Some(FieldValue::Single(
      value
    )) => {
      assert_eq!(
        value,
        "article-journal"
      );
    }
    | other => {
      panic!(
        "Expected FieldValue::Single, \
         got {other:?}"
      )
    }
  }
}

#[test]
fn parse_extracts_proc_natlsci_reference()
 {
  let parser = Parser::new();
  let references = parser.parse(
    &[BRISCHOUX_REF],
    ParseFormat::Json
  );

  let reference = &references[0].0;
  assert_list_field(
    reference,
    "journal",
    "Proc Natl Acad Sci U S A"
  );
  assert_list_field(
    reference, "volume", "106"
  );
  assert_list_field(
    reference,
    "pages",
    "4894-4899"
  );
  match reference.get("type") {
    | Some(FieldValue::Single(
      value
    )) => {
      assert_eq!(
        value,
        "article-journal"
      );
    }
    | other => {
      panic!(
        "Expected FieldValue::Single, \
         got {other:?}"
      )
    }
  }
}

#[test]
fn parse_extracts_han_reference() {
  let parser = Parser::new();
  let references = parser.parse(
    &[HAN_REF],
    ParseFormat::Json
  );

  let reference = &references[0].0;
  assert_list_field(
    reference,
    "journal",
    "Nano Lett"
  );
  assert_list_field(
    reference, "volume", "10"
  );
  assert_list_field(
    reference, "pages", "1012"
  );
  assert!(
    !reference.contains_key("genre"),
    "citation numbers should not be \
     treated as genre"
  );
  match reference.get("citation-number")
  {
    | Some(FieldValue::Single(
      value
    )) => {
      assert_eq!(value, "13");
    }
    | other => {
      panic!(
        "Expected FieldValue::Single, \
         got {other:?}"
      )
    }
  }
}

#[test]
fn label_handles_empty_lines() {
  let parser = Parser::new();
  assert!(parser.label("").is_empty());
  assert!(
    parser.label("\n").is_empty()
  );
  assert!(
    parser.label(" \n \n").is_empty()
  );
}

#[test]
fn label_outputs_all_expected_segment_types()
 {
  let parser = Parser::new();
  let sequences =
    parser.label(&format!(
      "{}\n{}",
      PEREC_REF, PEREC_REF
    ));

  let found: Vec<String> = sequences
    .iter()
    .flatten()
    .map(|token| token.label.clone())
    .collect();

  let unique_labels: Vec<_> = found
    .into_iter()
    .collect::<BTreeSet<_>>()
    .into_iter()
    .collect();

  let expected_labels = [
    "author",
    "title",
    "location",
    "publisher",
    "date",
    "pages"
  ];
  for expected in expected_labels {
    assert!(
      unique_labels.contains(
        &expected.to_string()
      ),
      "label output should contain \
       {expected}"
    );
  }
}

#[test]
fn label_handles_unrecognizable_input()
{
  let parser = Parser::new();
  parser
    .label("@misc{70213094902020,\n");
  parser.label("\n doi ");
}

fn assert_list_field(
  reference: &BTreeMap<
    String,
    FieldValue
  >,
  key: &str,
  expected: &str
) {
  match reference.get(key) {
    | Some(FieldValue::List(
      values
    )) => {
      assert!(
        values.iter().any(|value| {
          value == expected
        }),
        "field {key} should contain \
         {expected}"
      );
    }
    | other => {
      panic!(
        "expected list value for \
         {key}, got {other:?}"
      )
    }
  }
}
