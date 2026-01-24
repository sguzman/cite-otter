use std::collections::BTreeSet;

use unicode_script::{
  Script,
  UnicodeScript
};
use whatlang::{
  Lang,
  detect
};

pub fn detect_language(
  text: &str
) -> String {
  detect(text)
    .and_then(|info| {
      map_lang(info.lang())
    })
    .map(|code| code.to_string())
    .unwrap_or_else(|| {
      fallback_language(text)
    })
}

pub fn detect_scripts(
  text: &str
) -> Vec<String> {
  let mut scripts = BTreeSet::new();
  let mut has_latin = false;
  scripts.insert("Common".to_string());

  for ch in text.chars() {
    let script = ch.script();
    match script {
      | Script::Latin => {
        has_latin = true;
        scripts.insert(
          script
            .full_name()
            .to_string()
        );
      }
      | Script::Common
      | Script::Inherited
      | Script::Unknown => {}
      | _ => {
        scripts.insert(
          script
            .full_name()
            .to_string()
        );
      }
    }
  }

  if has_latin
    || text
      .chars()
      .any(|c| c.is_ascii_alphabetic())
  {
    scripts.insert("Latin".to_string());
  }

  scripts.into_iter().collect()
}

fn fallback_language(
  text: &str
) -> String {
  if text
    .chars()
    .any(|c| c.is_ascii_alphabetic())
  {
    "en".to_string()
  } else {
    "und".to_string()
  }
}

fn map_lang(
  lang: Lang
) -> Option<&'static str> {
  match lang {
    | Lang::Ara => Some("ar"),
    | Lang::Ces => Some("cs"),
    | Lang::Dan => Some("da"),
    | Lang::Deu => Some("de"),
    | Lang::Ell => Some("el"),
    | Lang::Eng => Some("en"),
    | Lang::Fin => Some("fi"),
    | Lang::Fra => Some("fr"),
    | Lang::Heb => Some("he"),
    | Lang::Hin => Some("hi"),
    | Lang::Hun => Some("hu"),
    | Lang::Ita => Some("it"),
    | Lang::Jpn => Some("ja"),
    | Lang::Kor => Some("ko"),
    | Lang::Nld => Some("nl"),
    | Lang::Nob => Some("no"),
    | Lang::Pol => Some("pl"),
    | Lang::Por => Some("pt"),
    | Lang::Rus => Some("ru"),
    | Lang::Spa => Some("es"),
    | Lang::Swe => Some("sv"),
    | Lang::Tur => Some("tr"),
    | Lang::Ukr => Some("uk"),
    | Lang::Cmn => Some("zh"),
    | _ => None
  }
}
