pub mod vocab {
    use csv::StringRecord;
    use serde_derive::{Deserialize as De, Serialize as Ser};
    use std::collections::HashMap;
    use std::error::Error;
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::BufReader;

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

    pub fn parse() -> Result<HashMap<String, Item>, Box<dyn Error>> {
        let f = File::open("raw/Spoonfed Pekzep - 語彙整理（超草案）.tsv")?;
        let f = BufReader::new(f);
        let mut res = HashMap::new();
        let mut errors = vec![];
        for line in f.lines() {
            // to prevent double quotes from vanishing, I do not read with CSV parser
            let row: Record = StringRecord::from(line.unwrap().split('\t').collect::<Vec<_>>())
                .deserialize(None)?;
            if !row.key.is_empty()
                && res
                    .insert(
                        row.key.clone(),
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
            let err: Box<dyn Error> = errors.join("\n").into();
            Err(err)
        }
    }
}

pub mod phrase {
    use csv::StringRecord;
    use linked_hash_map::LinkedHashMap;
    use partition_eithers::collect_any_errors;
    use pekzep_syllable::PekZepSyllable;
    use serde_derive::{Deserialize as De, Serialize as Ser};
    use std::collections::HashSet;
    use std::error::Error;
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::BufReader;

    #[derive(Debug, Clone, Eq, PartialEq, Hash, Copy)]
    pub enum ExtSyllable {
        Syllable(PekZepSyllable),
        Xizi,
    }

    impl std::fmt::Display for ExtSyllable {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                ExtSyllable::Syllable(s) => write!(f, "{}", s),
                ExtSyllable::Xizi => write!(f, "xizi"),
            }
        }
    }

    impl ExtSyllable {
        fn to_rerrliratixka(self) -> String {
            match &self {
                ExtSyllable::Syllable(s) => s.clone().to_rerrliratixka(),
                ExtSyllable::Xizi => "xizi".to_string(),
            }
        }
    }

    pub fn syllables_to_rerrliratixka_no_space(syllables: &[ExtSyllable]) -> String {
        syllables
            .iter()
            .map(|a| a.to_rerrliratixka())
            .collect::<Vec<_>>()
            .join("")
    }

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
    }

    #[readonly::make]
    #[derive(Debug, Clone)]
    pub struct Item {
        pub english: String,
        pub pekzep_latin: String,
        pub pekzep_hanzi: String,
        pub chinese_pinyin: String,
        pub chinese_hanzi: String,
        pub decomposed: String,
        pub filetype: HashSet<FilePathType>,
        pub recording_author: Option<Author>,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum FilePathType {
        Wav,
        WavR,
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
                Author::JektoVatimeliju => write!(f, "jekto.vatimeliju"),
                Author::FaliraLyjotafis => write!(f, "falira.lyjotafis"),
                Author::Other(a) => write!(f, "{}", a),
            }
        }
    }

    fn encode_to_pekzep_syllables(i: &str) -> Result<Vec<ExtSyllable>, Box<dyn Error>> {
        collect_any_errors(
            i.split(|c: char| c.is_ascii_punctuation() || c.is_whitespace())
                .filter_map(|k| {
                    if k.is_empty() {
                        None
                    } else {
                        Some(match PekZepSyllable::parse(k) {
                            Some(s) => Ok(ExtSyllable::Syllable(s)),
                            None => {
                                if k == "xizi" {
                                    Ok(ExtSyllable::Xizi)
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

    pub fn parse() -> Result<LinkedHashMap<Vec<ExtSyllable>, Item>, Box<dyn Error>> {
        use log::info;
        let f = File::open("raw/Spoonfed Pekzep - SpoonfedPekzep.tsv")?;
        let f = BufReader::new(f);
        let mut rows = LinkedHashMap::new();
        let mut errors = vec![];
        for line in f.lines() {
            // to prevent double quotes from vanishing, I do not read with CSV parser
            let rec: Record = StringRecord::from(line.unwrap().split('\t').collect::<Vec<_>>())
                .deserialize(None)?;

            info!("Parsing `{}`, `{}`:", rec.english, rec.pekzep_latin);
            let row = Item {
                pekzep_latin: rec.pekzep_latin,
                pekzep_hanzi: rec.pekzep_hanzi,
                chinese_hanzi: rec.chinese_hanzi,
                chinese_pinyin: rec.chinese_pinyin,
                english: rec.english,
                filetype: if rec.filetype.is_empty() {
                    HashSet::new()
                } else {
                    rec.filetype
                        .split(',')
                        .map(|x| match x.trim() {
                            "wav_r" => FilePathType::WavR,
                            "wav" => FilePathType::Wav,
                            "oga" => FilePathType::Oga,
                            a => panic!("Invalid file type `{}`. Run with RUST_LOG environment variable set to `info` to see the details.", a),
                        })
                        .collect()
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
                decomposed: rec.decomposed,
            };

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
            let err: Box<dyn Error> = errors.join("\n").into();
            Err(err)
        }
    }
}

pub mod char_pronunciation {
    use partition_eithers::collect_any_errors;
    use pekzep_syllable::PekZepSyllable;
    use serde_derive::Deserialize as De;
    use std::collections::HashMap;
    use std::error::Error;
    use std::fs::File;

    #[derive(Debug, De)]
    struct Record {
        character: String,
        sound: String,
        variant_of: String,
    }

    pub type CharSoundTable = Vec<(String, PekZepSyllable)>;
    pub type NonRecommendedCharTable = HashMap<String, String>;

    pub fn parse() -> Result<(CharSoundTable, NonRecommendedCharTable), Box<dyn Error>> {
        fn convert(record: &Record) -> Result<(String, PekZepSyllable), String> {
            match PekZepSyllable::parse(&record.sound) {
                None => Err(format!("Invalid sound {}", record.sound)),
                Some(a) => Ok((record.character.clone(), a)),
            }
        }

        let f = File::open("raw/字音.tsv")?;
        let mut rdr = csv::ReaderBuilder::new().delimiter(b'\t').from_reader(f);
        let mut ans = vec![];
        for result in rdr.deserialize() {
            let record: Record = result?;
            ans.push(record)
        }

        let a: Result<Vec<(String, PekZepSyllable)>, Box<dyn Error>> =
            collect_any_errors(ans.iter().map(convert).collect::<Vec<_>>())
                .map_err(|e| e.join("\n").into());

        let a: Vec<(String, PekZepSyllable)> = a?;

        let b = ans
            .iter()
            .filter_map(|r| {
                if r.variant_of.is_empty() {
                    None
                } else {
                    Some((r.character.clone(), r.variant_of.clone()))
                }
            })
            .collect::<HashMap<_, _>>();

        Ok((a, b))
    }
}
