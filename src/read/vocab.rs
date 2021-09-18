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
pub struct GlossVocab(String);

impl GlossVocab {
    pub fn new(a: &str) -> Self {
        /* FIXME: check more strictly */
        Self(a.to_owned())
    }

    pub fn to_internal_key(
        &self,
    ) -> anyhow::Result<(VocabInternalKey, Option<SplittableCompoundInfo>)> {
        let key = self.0.to_string().replace("!", " // ").replace("#", " // ");
        let splittable_compound_info = if self.0.contains('!') {
            Some(SplittableCompoundInfo::LatterHalfExclamation)
        } else if self.0.contains('#') {
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
/// It must adhere to one of the following formats (Note that, as of 2021-09-18, Note that we only allow CJK Unified Ideographs or CJK Unified Ideographs Extension A to be used as a transcription):
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
    raw: String,
    main: String,
    postfix: String,
}

impl VocabInternalKey {
    /// `享 // 銭` → `享_slashslash_銭`
    #[must_use]
    pub fn to_path_safe_string(&self) -> String {
        self.raw.replace(" // ", "_slashslash_")
    }

    #[must_use]
    pub fn into_string(self) -> String {
        self.raw
    }

    pub fn to_str(&self) -> &str {
        &self.raw
    }

    #[must_use]
    fn new(a: &str) -> anyhow::Result<Self> {
        /* FIXME: be more strict */
        Ok(Self {
            raw: a.to_owned(),
            main: a.to_owned(),
            postfix: a.to_owned(),
        })
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
