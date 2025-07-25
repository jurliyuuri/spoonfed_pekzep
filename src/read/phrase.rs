use crate::read::vocab::InternalKeyGloss;
use anyhow::anyhow;
use csv::StringRecord;
use linked_hash_map::LinkedHashMap;
use partition_eithers::collect_any_errors;
use pekzep_syllable::PekZepSyllable;
use serde_derive::{Deserialize as De, Serialize as Ser};
use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Copy)]
pub enum ExtSyllable {
    Syllable(PekZepSyllable),
    Xizi,
}

impl std::fmt::Display for ExtSyllable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Syllable(s) => write!(f, "{s}"),
            Self::Xizi => write!(f, "xizi"),
        }
    }
}

#[must_use]
pub fn syllables_to_str_underscore(syllables: &[ExtSyllable]) -> String {
    syllables
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>()
        .join("_")
}

#[readonly::make]
#[derive(Ser, De, Debug, Clone)]
pub struct Record {
    pub english: String,
    pub pekzep_latin: String,
    pub pekzep_hanzi: String,
    pub chinese_pinyin: String,
    pub chinese_hanzi: String,
    pub decomposed: String,
    pub filetype: String,
    pub recording_author: String,
    pub requires_substitution: String,
    pub japanese: String,
}

#[derive(Debug, Clone)]
pub struct SentenceGloss(pub Vec<InternalKeyGloss>);

impl SentenceGloss {
    pub fn to_plaintext(&self) -> String {
        self.0
            .iter()
            .map(InternalKeyGloss::to_plaintext)
            .collect::<String>()
    }
    pub fn to_debugtext(&self) -> String {
        self.0
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(".")
    }
}

#[readonly::make]
#[derive(Debug, Clone)]
pub struct Item {
    pub english: String,
    pub pekzep_latin: String,
    pub pekzep_hanzi: String,
    pub chinese_pinyin: String,
    pub chinese_hanzi: String,
    pub decomposed: Vec<SentenceGloss>,
    pub filetype: HashSet<FilePathType>,
    pub recording_author: Option<Author>,
    pub japanese: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FilePathType {
    Wav,
    Oga,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Author {
    JektoVatimeliju,
    FaliraLyjotafis,
    Other(String),
}

impl std::fmt::Display for Author {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::JektoVatimeliju => write!(f, "jekto.vatimeliju"),
            Self::FaliraLyjotafis => write!(f, "falira.lyjotafis"),
            Self::Other(a) => write!(f, "{a}"),
        }
    }
}

fn encode_to_pekzep_syllables(i: &str) -> anyhow::Result<Vec<ExtSyllable>> {
    collect_any_errors(
        i.split(|c: char| c.is_ascii_punctuation() || c.is_whitespace())
            .filter_map(|k| {
                if k.is_empty() {
                    None
                } else {
                    Some(PekZepSyllable::parse(k).map_or_else(
                        || {
                            if k == "xizi" {
                                Ok(ExtSyllable::Xizi)
                            } else {
                                Err(format!("Failed to parse a pekzep syllable {k}"))
                            }
                        },
                        |s| Ok(ExtSyllable::Syllable(s)),
                    ))
                }
            })
            .collect::<Vec<_>>(),
    )
    .map_err(|e| anyhow!(e.join("\n")))
}

#[allow(clippy::tabs_in_doc_comments)]
/// Parses "raw/Spoonfed Pekzep - SpoonfedPekzep.tsv" to obtain a table converting a string of characters to a contracted syllable.
/// The tsv used for the input should be of the following form:
/// ```text
///Hello / how are you	kait kia1!	善日！	Nǐ hǎo!	你好！	善日	wav	falira.lyjotafis		Jeemusn!
///I'm hurrying to work.	pai2 sam1 mok1 ie naip2 hue.	我急行於労処。	Wǒ cōngmáng de qù shàngbān.	我匆忙地去上班。	我.急行2.於1.労処
/// ```
/// # Errors
/// Gives errors if:
/// - IO fails
/// - Pekzep is invalid
/// - there is a duplication in phrases
/// - filetype is invalid
///
pub fn parse() -> anyhow::Result<LinkedHashMap<Vec<ExtSyllable>, Item>> {
    use log::info;
    let f = File::open("raw/Spoonfed Pekzep - SpoonfedPekzep.tsv")?;
    let f = BufReader::new(f);
    let mut rows = LinkedHashMap::new();
    let mut errors = vec![];
    for line in f.lines() {
        // to prevent double quotes from vanishing, I do not read with CSV parser
        let rec: Record =
            StringRecord::from(line?.split('\t').collect::<Vec<_>>()).deserialize(None)?;

        info!("Parsing `{}`, `{}`:", rec.english, rec.pekzep_latin);
        let decomposed = if rec.decomposed.is_empty() {
            vec![]
        } else {
            let sentences = rec.decomposed.split("..");
            let mut ans = vec![];
            for s in sentences {
                ans.push(SentenceGloss(
                    s.split('.')
                        .map(InternalKeyGloss::new)
                        .collect::<anyhow::Result<_>>()?,
                ));
            }
            ans
        };
        let row = Item {
            pekzep_latin: rec.pekzep_latin,
            pekzep_hanzi: rec.pekzep_hanzi,
            chinese_hanzi: rec.chinese_hanzi,
            chinese_pinyin: rec.chinese_pinyin,
            english: rec.english,
            japanese: rec.japanese,
            filetype: if rec.filetype.is_empty() {
                HashSet::new()
            } else {
                let filetypes = rec.filetype.split(',').collect::<Vec<_>>();
                let mut ans = HashSet::new();
                for x in filetypes {
                    ans.insert(match x.trim() {
                        "wav" => FilePathType::Wav,
                        "oga" => FilePathType::Oga,
                        a => return Err(anyhow!("Invalid file type `{}`. Run with RUST_LOG environment variable set to `info` to see the details.", a)),
                    });
                }
                ans
            },
            recording_author: if rec.recording_author == "jekto.vatimeliju" {
                Some(Author::JektoVatimeliju)
            } else if rec.recording_author == "falira.lyjotafis" {
                Some(Author::FaliraLyjotafis)
            } else if rec.recording_author.is_empty() {
                None
            } else {
                Some(Author::Other(rec.recording_author))
            },
            decomposed,
        };

        // 未査読の行は飛ばす
        if row.pekzep_hanzi.contains('@') {
            info!("`{}` is not yet peer-reviewed. Skipping.", row.pekzep_latin);
            continue;
        }

        let syllables = encode_to_pekzep_syllables(&row.pekzep_latin)?;
        if !syllables.is_empty() && rows.insert(syllables.clone(), row.clone()).is_some() {
            // in HashSet::insert, if the set did have this value present, false is returned.
            errors.push(format!(
                "duplicate phrase detected: {}",
                syllables_to_str_underscore(&syllables)
            ));
        }
    }

    if errors.is_empty() {
        Ok(rows)
    } else {
        let err = errors.join("\n");
        Err(anyhow!(err))
    }
}
