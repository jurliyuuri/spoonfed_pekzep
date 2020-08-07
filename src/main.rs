use csv::StringRecord;
use pekzep_syllable::PekZepSyllable;
use serde_derive::{Deserialize as De, Serialize as Ser};
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

#[derive(Ser, De, Debug, Clone)]
struct VocabRow {
    key: String,
    pekzep_latin: String,
    pekzep_hanzi: String,
    parts_of_speech: String,
    parts_of_speech_supplement: String,
    english_gloss: String,
}

#[derive(Debug, Clone)]
struct Vocab {
    pekzep_latin: String,
    pekzep_hanzi: String,
    parts_of_speech: String,
    parts_of_speech_supplement: String,
    english_gloss: String,
}

impl Vocab {
    pub fn to_tab_separated(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}\t{}",
            self.pekzep_latin,
            self.pekzep_hanzi,
            self.parts_of_speech,
            self.parts_of_speech_supplement,
            self.english_gloss
        )
    }
}

#[derive(Ser, De, Debug, Clone)]
struct MainRow {
    english: String,
    pekzep_latin: String,
    pekzep_hanzi: String,
    chinese_pinyin: String,
    chinese_hanzi: String,
    decomposed: String,
    lineparine: String,
}

use std::collections::HashMap;
use std::collections::HashSet;

fn parse_vocabs() -> Result<HashMap<String, Vocab>, Box<dyn Error>> {
    let f = File::open("raw/Spoonfed Pekzep - 語彙整理（超草案）.tsv")?;
    let f = BufReader::new(f);
    let mut res = HashMap::new();
    for line in f.lines() {
        // to prevent double quotes from vanishing, I do not read with CSV parser
        let row: VocabRow =
            StringRecord::from(line.unwrap().split('\t').collect::<Vec<_>>()).deserialize(None)?;
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
            panic!("duplicate key detected: {}", row.key);
        }
    }
    Ok(res)
}

fn parse_spoonfed() -> Result<Vec<MainRow>, Box<dyn Error>> {
    let f = File::open("raw/Spoonfed Pekzep - SpoonfedPekzep.tsv")?;
    let f = BufReader::new(f);

    let mut rows = vec![];
    let mut detect_dup_in_pekzep = HashSet::new();
    for line in f.lines() {
        // to prevent double quotes from vanishing, I do not read with CSV parser
        let row: MainRow =
            StringRecord::from(line.unwrap().split('\t').collect::<Vec<_>>()).deserialize(None)?;

        let url = encode_to_url(&row.pekzep_latin);
        if !url.is_empty() && !detect_dup_in_pekzep.insert(url.clone()) {
            // in HashSet::insert, if the set did have this value present, false is returned.
            panic!("duplicate entry detected: {}", url);
        }

        rows.push(row);
    }
    Ok(rows)
}

use askama::Template;

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate<'a> {
    english: &'a str,
    chinese_hanzi: &'a str,
    chinese_pinyin: &'a str,
    pekzep_latin: &'a str,
    pekzep_hanzi: &'a str,
    prev_link: &'a str,
    next_link: &'a str,
    audio_path: &'a str,
    analysis: &'a str,
}

mod filters {
    pub fn capitalizefirstchar(s: &str) -> ::askama::Result<String> {
        let mut v: Vec<char> = s.chars().collect();
        v[0] = v[0].to_uppercase().nth(0).unwrap();
        let s2: String = v.into_iter().collect();
        Ok(s2)
    }
    pub fn linebreaksandtabs(s: &str) -> ::askama::Result<String> {
        let s = s.to_string();
        Ok(format!(
            "<table border=1>\n\t<tr><td>{}</td></tr>\n<table>",
            s.replace("\t", "</td><td>")
                .replace("\n", "</td></tr>\n\t<tr><td>")
        ))
    }
}

fn encode_to_sound_path(i: &str) -> String {
    i.split(|c: char| c.is_ascii_punctuation() || c.is_whitespace())
        .filter(|a| !a.is_empty())
        .map(|k| match PekZepSyllable::parse(k) {
            Some(s) => s.to_rerrliratixka(),
            None => {
                if k == "xizi" {
                    "xizi".to_string()
                } else {
                    panic!("Failed to parse a pekzep syllable {}", k)
                }
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

fn encode_to_url(i: &str) -> String {
    i.split(|c: char| c.is_ascii_punctuation() || c.is_whitespace())
        .filter(|a| !a.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

fn link_url(prev: &Option<MainRow>) -> String {
    match prev {
        None => "index".to_string(),
        Some(p) => {
            if p.pekzep_latin.is_empty() {
                "index".to_string()
            } else {
                encode_to_url(&p.pekzep_latin)
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let spoonfed_rows = parse_spoonfed()?;

    let vocab = parse_vocabs()?;

    let mut rows2: Vec<Option<MainRow>> =
        spoonfed_rows.clone().into_iter().map(|r| Some(r)).collect();
    rows2.push(None);
    rows2.insert(0, None);

    for v in rows2.windows(3) {
        match v {
            [prev, Some(this), next] => {
                if this.pekzep_latin.is_empty() {
                    continue;
                }
                let mut file =
                    File::create(format!("docs/{}.html", encode_to_url(&this.pekzep_latin)))?;
                let analysis = if this.decomposed.is_empty() {
                    vec![]
                } else {
                    this
                    .decomposed.split('.')
                    .map(|a| {
                        let key = a.to_string().replace("!", " // ").replace("#", " // ");
                        let res = vocab.get(&key).expect(&format!(
                            "Cannot find key {} in the vocab list, found while analyzing {} (len {})",
                            key, this.decomposed, this.decomposed.len()
                        ));
                        res.to_tab_separated()
                    })
                    .collect::<Vec<_>>()
                };
                let hello = HelloTemplate {
                    english: &this.english,
                    chinese_pinyin: &this.chinese_pinyin,
                    chinese_hanzi: &this.chinese_hanzi,
                    pekzep_latin: &this.pekzep_latin,
                    pekzep_hanzi: &this.pekzep_hanzi,
                    prev_link: &link_url(prev),
                    next_link: &link_url(next),
                    audio_path: &encode_to_sound_path(&this.pekzep_latin),
                    analysis: &analysis.join("\n"),
                };
                write!(file, "{}", hello.render().unwrap())?;
            }
            _ => unreachable!(),
        }
    }

    let mut file = File::create("docs/index.html")?;
    let mut index =
        r#"<!doctype HTML><html><head><meta charset="UTF-8"></head><body><h1>Spoonfed Pekzep</h1>"#
            .to_string();
    for r in spoonfed_rows {
        if r.pekzep_latin.is_empty() {
            index.push_str("*<br>");
        } else {
            index.push_str(&format!(
                "<a href=\"{}.html\">{}</a><br>\n",
                encode_to_url(&r.pekzep_latin),
                r.pekzep_latin
            ));
        }
    }
    index.push_str("</body></html>");
    write!(file, "{}", index)?;

    Ok(())
}
