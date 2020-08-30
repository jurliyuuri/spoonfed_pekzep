pub mod vocab {
    use csv::StringRecord;
    use serde_derive::{Deserialize as De, Serialize as Ser};
    use std::collections::HashMap;
    use std::error::Error;
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::BufReader;

    #[derive(Ser, De, Debug, Clone)]
    pub struct VocabRow {
        key: String,
        pekzep_latin: String,
        pekzep_hanzi: String,
        parts_of_speech: String,
        parts_of_speech_supplement: String,
        english_gloss: String,
    }

    #[derive(Debug, Clone)]
    pub struct Vocab {
        pekzep_latin: String,
        pekzep_hanzi: String,
        parts_of_speech: String,
        parts_of_speech_supplement: String,
        english_gloss: String,
    }

    impl Vocab {
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

    pub fn parse_vocabs() -> Result<HashMap<String, Vocab>, Box<dyn Error>> {
        let f = File::open("raw/Spoonfed Pekzep - 語彙整理（超草案）.tsv")?;
        let f = BufReader::new(f);
        let mut res = HashMap::new();
        let mut errors = vec![];
        for line in f.lines() {
            // to prevent double quotes from vanishing, I do not read with CSV parser
            let row: VocabRow = StringRecord::from(line.unwrap().split('\t').collect::<Vec<_>>())
                .deserialize(None)?;
            if !row.key.is_empty()
                && res
                    .insert(
                        row.key.clone(),
                        Vocab {
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
            let err: Box<dyn Error> = errors.join("\n").into();
            Err(err)
        }
    }
}

pub mod main_row {
    use csv::StringRecord;
    use linked_hash_map::LinkedHashMap;
    use partition_eithers::collect_any_errors;
    use pekzep_syllable::PekZepSyllable;
    use serde_derive::{Deserialize as De, Serialize as Ser};
    use std::error::Error;
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::BufReader;

    #[derive(Debug, Clone, Eq, PartialEq, Hash)]
    pub enum ExtSyll {
        Syll(PekZepSyllable),
        Xizi,
    }

    impl std::fmt::Display for ExtSyll {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                ExtSyll::Syll(s) => write!(f, "{}", s),
                ExtSyll::Xizi => write!(f, "xizi"),
            }
        }
    }

    impl ExtSyll {
        fn to_rerrliratixka(&self) -> String {
            match &self {
                ExtSyll::Syll(s) => s.clone().to_rerrliratixka(),
                ExtSyll::Xizi => "xizi".to_string(),
            }
        }
    }

    pub fn sylls_to_rerrliratixka_no_space(sylls: &[ExtSyll]) -> String {
        sylls
            .iter()
            .map(ExtSyll::to_rerrliratixka)
            .collect::<Vec<_>>()
            .join("")
    }

    pub fn sylls_to_str_underscore(sylls: &[ExtSyll]) -> String {
        sylls
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join("_")
    }

    #[derive(Ser, De, Debug, Clone)]
    pub struct MainRow {
        pub english: String,
        pub pekzep_latin: String,
        pub pekzep_hanzi: String,
        pub chinese_pinyin: String,
        pub chinese_hanzi: String,
        pub decomposed: String,
        pub filetype: String,
        pub recording_author: String,
    }

    fn encode_to_pekzep_syllables(i: &str) -> Result<Vec<ExtSyll>, Box<dyn Error>> {
        collect_any_errors(
            i.split(|c: char| c.is_ascii_punctuation() || c.is_whitespace())
                .filter_map(|k| {
                    if k.is_empty() {
                        None
                    } else {
                        Some(match PekZepSyllable::parse(k) {
                            Some(s) => Ok(ExtSyll::Syll(s)),
                            None => {
                                if k == "xizi" {
                                    Ok(ExtSyll::Xizi)
                                } else {
                                    Err(format!("Failed to parse a pekzep syllable {}", k))
                                }
                            }
                        })
                    }
                })
                .collect::<Vec<_>>(),
        )
        .map_err(|e| e.join("\n").into())
    }

    pub fn parse_spoonfed() -> Result<LinkedHashMap<Vec<ExtSyll>, MainRow>, Box<dyn Error>> {
        let f = File::open("raw/Spoonfed Pekzep - SpoonfedPekzep.tsv")?;
        let f = BufReader::new(f);
        let mut rows = LinkedHashMap::new();
        let mut errors = vec![];
        for line in f.lines() {
            // to prevent double quotes from vanishing, I do not read with CSV parser
            let row: MainRow = StringRecord::from(line.unwrap().split('\t').collect::<Vec<_>>())
                .deserialize(None)?;

            let sylls = encode_to_pekzep_syllables(&row.pekzep_latin)?;
            if !sylls.is_empty() && rows.insert(sylls.clone(), row.clone()).is_some() {
                // in HashSet::insert, if the set did have this value present, false is returned.
                errors.push(format!(
                    "duplicate phrase detected: {}",
                    sylls_to_str_underscore(&sylls)
                ));
            }
        }

        if errors.is_empty() {
            Ok(rows)
        } else {
            let err: Box<dyn Error> = errors.join("\n").into();
            Err(err)
        }
    }
}
