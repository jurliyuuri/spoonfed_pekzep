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
