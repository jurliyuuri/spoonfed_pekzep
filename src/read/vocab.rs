use anyhow::anyhow;
use csv::StringRecord;
use serde_derive::{Deserialize as De, Serialize as Ser};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]

/// an enum used to talk about a splittable compound
pub enum SplittableCompoundInfo {
    /// Denotes the former half of the splittable compound, such as `享#銭`
    FormerHalfHash,

    /// Denotes the latter half of the splittable compound, such as `享!銭`
    LatterHalfExclamation,
}

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
            "{}\t{}\t<span style=\"filter:brightness(65%) contrast(500%);\">{}</span>\t{}\t{}\t{}",
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
/// Almost the same as `InternalKey`, but instead of `享 // 銭`, it uses `享#銭` to denote the former half and `享!銭` to denote the latter half of the splittable compound.
pub struct InternalKeyGloss {
    main: String,
    postfix: String,
}

impl std::fmt::Display for InternalKeyGloss {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}{}", self.main, self.postfix)
    }
}

impl InternalKeyGloss {
    /// Splits the input and stores it as the "main" and the "postfix".
    /// # Errors
    /// Returns error if the input does not start with one of the following:
    /// - an ASCII character
    /// - a character in the Unicode block "CJK Unified Ideographs"
    /// - a character in the Unicode block "CJK Unified Ideographs Extension A"
    pub fn new(input: &str) -> anyhow::Result<Self> {
        let (main, postfix) = split_into_main_and_postfix(input)?;
        Ok(Self { main, postfix })
    }

    #[must_use]
    pub fn to_internal_key(&self) -> InternalKey {
        InternalKey {
            postfix: self.postfix.clone(),
            main: self.main.replace('!', " // ").replace('#', " // "),
        }
    }

    #[must_use]
    pub fn to_splittable_compound_info(&self) -> Option<SplittableCompoundInfo> {
        if self.to_string().contains('!') {
            Some(SplittableCompoundInfo::LatterHalfExclamation)
        } else if self.to_string().contains('#') {
            Some(SplittableCompoundInfo::FormerHalfHash)
        } else {
            None
        }
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
pub struct InternalKey {
    main: String,
    postfix: String,
}

mod tests {
    #[test]
    fn test_new() {
        use crate::read::vocab::InternalKey;
        use big_s::S;
        assert_eq!(
            InternalKey::new("於dur").unwrap(),
            InternalKey {
                postfix: S("dur"),
                main: S("於")
            }
        );
        assert_eq!(
            InternalKey::new("於").unwrap(),
            InternalKey {
                postfix: S(""),
                main: S("於")
            }
        );
        assert_eq!(
            InternalKey::new("xizi:375").unwrap(),
            InternalKey {
                postfix: S(":375"),
                main: S("xizi")
            }
        );
        assert_eq!(
            InternalKey::new("xizi").unwrap(),
            InternalKey {
                postfix: S(""),
                main: S("xizi")
            }
        );
        assert_eq!(
            InternalKey::new("xizi375").unwrap(),
            InternalKey {
                postfix: S(""),
                main: S("xizi375")
            }
        );
    }

    #[test]
    fn test_new2() {
        use crate::read::vocab::InternalKeyGloss;
        use big_s::S;
        assert_eq!(
            InternalKeyGloss::new("於dur").unwrap(),
            InternalKeyGloss {
                postfix: S("dur"),
                main: S("於")
            }
        );
        assert_eq!(
            InternalKeyGloss::new("於").unwrap(),
            InternalKeyGloss {
                postfix: S(""),
                main: S("於")
            }
        );
        assert_eq!(
            InternalKeyGloss::new("xizi:375").unwrap(),
            InternalKeyGloss {
                postfix: S(":375"),
                main: S("xizi")
            }
        );
        assert_eq!(
            InternalKeyGloss::new("xizi").unwrap(),
            InternalKeyGloss {
                postfix: S(""),
                main: S("xizi")
            }
        );
        assert_eq!(
            InternalKeyGloss::new("xizi375").unwrap(),
            InternalKeyGloss {
                postfix: S(""),
                main: S("xizi375")
            }
        );
    }

    #[test]
    fn test_to_path_safe_string() {
        use crate::read::vocab::InternalKey;
        assert_eq!(
            InternalKey::new("xizi375").unwrap().to_path_safe_string(),
            "xizi375"
        );

        assert_eq!(
            InternalKey::new("享 // 銭").unwrap().to_path_safe_string(),
            "享_slashslash_銭"
        );

        assert_eq!(
            InternalKey::new("識 // 言2").unwrap().to_path_safe_string(),
            "識_slashslash_言2"
        );
    }
}

impl std::fmt::Display for InternalKey {
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

        '\u{3400}'..='\u{4DBF}' | '\u{4E00}'..='\u{9FFF}' | '∅' | '«' => {
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
- `∅`
- `«`
- a character in the Unicode block \"CJK Unified Ideographs\"
- a character in the Unicode block \"CJK Unified Ideographs Extension A\"",
            input
        ))
        }
    };
    Ok((main, postfix))
}

impl InternalKey {
    /// `享 // 銭` → `享_slashslash_銭`
    #[must_use]
    pub fn to_path_safe_string(&self) -> String {
        self.to_string()
            .replace(" // ", "_slashslash_")
            .replace(':', "_colon_")
    }

    fn new(input: &str) -> anyhow::Result<Self> {
        let (main, postfix) = split_into_main_and_postfix(input)?;
        Ok(Self { main, postfix })
    }
}

#[allow(clippy::tabs_in_doc_comments)]
/// Parses "raw/Spoonfed Pekzep - 語彙整理（超草案）.tsv" to make a lookup table from the `InternalKey` to the `Item`.
/// The tsv used for the input should be of the following form:
/// ```text
///善日	kait kia1	善日	interjection	greeting	hello
///汝	mua2	汝	noun		you
///言	zep1	言	verb		to say
///何	nan2	何	noun	interrogative	what
///彼等	zap2 ge	彼等	noun		they
///在	aim2	在	verb-modifier	aspect marker	be ~ing
/// ```
/// # Errors
/// Gives errors if:
/// - IO fails
/// - "raw/Spoonfed Pekzep - 語彙整理（超草案）.tsv" does not conform to an expected format
///
pub fn parse() -> anyhow::Result<HashMap<InternalKey, Item>> {
    let f = File::open("raw/Spoonfed Pekzep - 語彙整理（超草案）.tsv")?;
    let f = BufReader::new(f);
    let mut res = HashMap::new();
    let mut errors = vec![];
    for line in f.lines() {
        // to prevent double quotes from vanishing, I do not read with CSV parser
        let row: Record =
            StringRecord::from(line?.split('\t').collect::<Vec<_>>()).deserialize(None)?;
        if !row.key.is_empty()
            && res
                .insert(
                    InternalKey::new(&row.key)?,
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
