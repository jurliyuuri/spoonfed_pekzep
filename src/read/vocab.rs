use anyhow::anyhow;
use csv::StringRecord;
use serde_derive::{Deserialize as De, Serialize as Ser};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use crate::verify::SplittableCompoundInfo;

#[derive(Ser, De, Debug, Clone)]
struct Record {
    key: String,
    pekzep_latin: String,
    pekzep_hanzi: String,
    parts_of_speech: String,
    parts_of_speech_supplement: String,
    english_gloss: String,
}

#[readonly::make]
#[derive(Debug, Clone)]
pub struct Item {
    pub pekzep_latin: String,
    pub pekzep_hanzi: String,
    pub parts_of_speech: String,
    pub parts_of_speech_supplement: String,
    pub english_gloss: String,
}

impl Item {
    pub fn to_tab_separated_with_custom_linzifier<F>(&self, f: F) -> String
    where
        F: FnOnce(&str) -> String,
    {
        format!(
            "{}\t{}\t<span style=\"filter:brightness(65%)contrast(500%);\">{}</span>\t{}\t{}\t{}",
            self.pekzep_latin,
            self.pekzep_hanzi,
            f(&self.pekzep_hanzi),
            self.parts_of_speech,
            self.parts_of_speech_supplement,
            self.english_gloss
        )
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
/// Almost the same as `VocabInternalKey`, but instead of `享 // 銭`, it uses `享#銭` to denote the former half and `享!銭` to denote the latter half of the splittable compound.
pub struct GlossVocab {
    main: String,
    postfix: String,
}

impl std::fmt::Display for GlossVocab {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}{}", self.main, self.postfix)
    }
}

impl GlossVocab {
    pub fn new(input: &str) -> anyhow::Result<Self> {
        let (main, postfix) = split_into_main_and_postfix(input)?;
        Ok(Self { main, postfix })
    }

    pub fn to_internal_key(
        &self,
    ) -> anyhow::Result<(VocabInternalKey, Option<SplittableCompoundInfo>)> {
        let key = self.to_string().replace("!", " // ").replace("#", " // ");
        let splittable_compound_info = if self.to_string().contains('!') {
            Some(SplittableCompoundInfo::LatterHalfExclamation)
        } else if self.to_string().contains('#') {
            Some(SplittableCompoundInfo::FormerHalfHash)
        } else {
            None
        };
        Ok((VocabInternalKey::new(&key)?, splittable_compound_info))
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
/// The key used to identify a word in the glosses. It is made up of a "main" followed by an optional "postfix", where "main" denotes the word and "postfix" disambiguates the subdivision of the word.
///
/// The postfix is `[0-9a-zA-Z]*` when the main does not begin with an ASCII character. The postfix is `:[0-9a-zA-Z]*` if the main *does* begin with an ASCII character.
///
/// It must adhere to one of the following formats (Note that, as of 2021-09-18, Note that we only allow chars in the Unicode block "CJK Unified Ideographs" or "CJK Unified Ideographs Extension A" to be used as a transcription):
///
/// | Main | Optional postfix                                                              | Annotation Removed                     | Example |                                                                                                                                                       |
/// |---------|----------------------------------------------------------------------|----------------------------------------|-------------|-------------------------------------------------------------------------------------------------------------------------------------------------------|
/// |  `[\u3400-\u4DBF\u4E00-\u9FFF]+`       | `[0-9a-zA-Z]*`                        | `種茶銭処`, `紙机戦`, `於dur`, `須多2` |  The most basic form available for a key: the postfix disambiguates which meaning is intended for the glossed word.                                            |
/// |  `∅`      | `[0-9a-zA-Z]*`                                                      | `∅`, `∅3`                              | used when the word is realized as an empty string in Pekzep                                                                                           |
/// |  `[\u3400-\u4DBF\u4E00-\u9FFF]+) // ([\u3400-\u4DBF\u4E00-\u9FFF]+`      | `[0-9a-zA-Z]*` | `享 // 銭`, `行 // 星周`               | used for a splittable compound                                                                                                                        |
/// |  `«[\u3400-\u4DBF\u4E00-\u9FFF]+»`     | `[0-9a-zA-Z]*`                                  | `«足手»`                               |  used when a multisyllable merges into a single syllable                                                                                               |
/// |  `[a-z0-9 ]+` (cannot start or end with a space)     | `:[0-9a-zA-Z]*`                                          | `xizi`, `xizi xizi`                    |  Denotes `xizi`, a postfix used after a name, or `xizi xizi`, an interjection. Currently, this program does not allow any non-Linzklar word other than `xizi`. |
/// |  `[a-z0-9 ]+[\u3400-\u4DBF\u4E00-\u9FFF]+`     | `:[0-9a-zA-Z]*`             | `xizi噫`                               |  Denotes `xizi噫`, an interjection. Currently, this program does not allow any non-Linzklar word other than `xizi`.                                    |
/// |  `\([\u3400-\u4DBF\u4E00-\u9FFF]+\)`     | `:[0-9a-zA-Z]*`                                 | `(噫)`                               |  used for the 噫 placed after 之 to mark that the sentence ends with a possessive                                                                                              |
pub struct VocabInternalKey {
    main: String,
    postfix: String,
}

mod tests {
    #[test]
    fn test_new() {
        use crate::read::vocab::VocabInternalKey;
        use big_s::S;
        assert_eq!(
            VocabInternalKey::new("於dur").unwrap(),
            VocabInternalKey {
                postfix: S("dur"),
                main: S("於")
            }
        );
        assert_eq!(
            VocabInternalKey::new("於").unwrap(),
            VocabInternalKey {
                postfix: S(""),
                main: S("於")
            }
        );
        assert_eq!(
            VocabInternalKey::new("xizi:375").unwrap(),
            VocabInternalKey {
                postfix: S(":375"),
                main: S("xizi")
            }
        );
        assert_eq!(
            VocabInternalKey::new("xizi").unwrap(),
            VocabInternalKey {
                postfix: S(""),
                main: S("xizi")
            }
        );
        assert_eq!(
            VocabInternalKey::new("xizi375").unwrap(),
            VocabInternalKey {
                postfix: S(""),
                main: S("xizi375")
            }
        );
    }

    fn test_new2() {
        use crate::read::vocab::GlossVocab;
        use big_s::S;
        assert_eq!(
            GlossVocab::new("於dur").unwrap(),
            GlossVocab {
                postfix: S("dur"),
                main: S("於")
            }
        );
        assert_eq!(
            GlossVocab::new("於").unwrap(),
            GlossVocab {
                postfix: S(""),
                main: S("於")
            }
        );
        assert_eq!(
            GlossVocab::new("xizi:375").unwrap(),
            GlossVocab {
                postfix: S(":375"),
                main: S("xizi")
            }
        );
        assert_eq!(
            GlossVocab::new("xizi").unwrap(),
            GlossVocab {
                postfix: S(""),
                main: S("xizi")
            }
        );
        assert_eq!(
            GlossVocab::new("xizi375").unwrap(),
            GlossVocab {
                postfix: S(""),
                main: S("xizi375")
            }
        );
    }

    #[test]
    fn test_to_path_safe_string() {
        use crate::read::vocab::VocabInternalKey;
        assert_eq!(
            VocabInternalKey::new("xizi375")
                .unwrap()
                .to_path_safe_string(),
            "xizi375"
        );

        assert_eq!(
            VocabInternalKey::new("享 // 銭")
                .unwrap()
                .to_path_safe_string(),
            "享_slashslash_銭"
        );
    }
}

impl std::fmt::Display for VocabInternalKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}{}", self.main, self.postfix)
    }
}

fn split_into_main_and_postfix(input: &str) -> anyhow::Result<(String, String)> {
    let (main, postfix) = match input
        .chars()
        .next()
        .ok_or_else(|| anyhow!("empty string encountered"))?
    {
        '!'..='~' => {
            // The postfix is `:[0-9a-zA-Z]*` if the main *does* begin with an ASCII character.
            let v: Vec<&str> = input.splitn(2, ':').collect();
            match v[..] {
                [main, postfix] => (main.to_owned(), format!(":{}", postfix)),
                [main] => (main.to_owned(), String::new()),
                _ => panic!("cannot happen"),
            }
        }

        '\u{3400}'..='\u{4DBF}' | '\u{4E00}'..='\u{9FFF}' => {
            // The postfix is `[0-9a-zA-Z]*` when the main does not begin with an ASCII character.
            let rev_main = input
                .chars()
                .rev()
                .skip_while(char::is_ascii_alphanumeric)
                .collect::<String>();
            let rev_postfix = input
                .chars()
                .rev()
                .take_while(char::is_ascii_alphanumeric)
                .collect::<String>();
            (
                rev_main.chars().rev().collect::<String>(),
                rev_postfix.chars().rev().collect::<String>(),
            )
        }
        _ => {
            return Err(anyhow!(
            "The input, `{}`, began with an unexpected character. It must begin either with either:
- an ASCII character
- a character in the Unicode block \"CJK Unified Ideographs\"
- a character in the Unicode block \"CJK Unified Ideographs Extension A\"",
            input
        ))
        }
    };
    Ok((main, postfix))
}

impl VocabInternalKey {
    /// `享 // 銭` → `享_slashslash_銭`
    #[must_use]
    pub fn to_path_safe_string(&self) -> String {
        self.to_string()
            .replace(" // ", "_slashslash_")
            .replace(":", "_colon_")
    }

    fn new(input: &str) -> anyhow::Result<Self> {
        let (main, postfix) = split_into_main_and_postfix(input)?;
        Ok(Self { main, postfix })
    }
}

pub fn parse() -> anyhow::Result<HashMap<VocabInternalKey, Item>> {
    let f = File::open("raw/Spoonfed Pekzep - 語彙整理（超草案）.tsv")?;
    let f = BufReader::new(f);
    let mut res = HashMap::new();
    let mut errors = vec![];
    for line in f.lines() {
        // to prevent double quotes from vanishing, I do not read with CSV parser
        let row: Record =
            StringRecord::from(line.unwrap().split('\t').collect::<Vec<_>>()).deserialize(None)?;
        if !row.key.is_empty()
            && res
                .insert(
                    VocabInternalKey::new(&row.key)?,
                    Item {
                        pekzep_latin: row.pekzep_latin,
                        pekzep_hanzi: row.pekzep_hanzi,
                        parts_of_speech: row.parts_of_speech,
                        parts_of_speech_supplement: row.parts_of_speech_supplement,
                        english_gloss: row.english_gloss,
                    },
                )
                .is_some()
        {
            errors.push(format!("duplicate key detected: {}", row.key));
        }
    }
    if errors.is_empty() {
        Ok(res)
    } else {
        let err = errors.join("\n");
        Err(anyhow!(err))
    }
}
