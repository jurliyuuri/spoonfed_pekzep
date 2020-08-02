use csv::StringRecord;
use serde_derive::{Deserialize as De, Serialize as Ser};
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

mod rerrliratixka;

#[derive(Ser, De, Debug, Clone)]
struct Row {
    english: String,
    pekzep_latin: String,
    pekzep_hanzi: String,
    chinese_pinyin: String,
    chinese_hanzi: String,
    lineparine: String,
}

fn parse_spoonfed() -> Result<Vec<Row>, Box<dyn Error>> {
    let f = File::open("raw/Spoonfed Pekzep - SpoonfedPekzep.tsv")?;
    let f = BufReader::new(f);

    let mut rows = vec![];
    for line in f.lines() {
        // to prevent double quotes from vanishing, I do not read with CSV parser
        let row: Row =
            StringRecord::from(line.unwrap().split('\t').collect::<Vec<_>>()).deserialize(None)?;
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
}

fn encode_to_sound_path(i: &str) -> String {
    i.split(|c: char| c.is_ascii_punctuation() || c.is_whitespace())
        .filter(|a| !a.is_empty())
        .map(|k| match rerrliratixka::PekZepSyllable::parse(k) {
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

fn link_url(prev: &Option<Row>) -> String {
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
    let rows = parse_spoonfed()?;

    let mut rows2: Vec<Option<Row>> = rows.clone().into_iter().map(|r| Some(r)).collect();
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
                let hello = HelloTemplate {
                    english: &this.english,
                    chinese_pinyin: &this.chinese_pinyin,
                    chinese_hanzi: &this.chinese_hanzi,
                    pekzep_latin: &this.pekzep_latin,
                    pekzep_hanzi: &this.pekzep_hanzi,
                    prev_link: &link_url(prev),
                    next_link: &link_url(next),
                    audio_path: &encode_to_sound_path(&this.pekzep_latin),
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
    for r in rows {
        if r.pekzep_latin.is_empty() {
            index.push_str("*<br>");
        } else {
            index.push_str(&format!(
                r#"<a href="{}.html">{}</a><br>"#,
                encode_to_url(&r.pekzep_latin),
                r.pekzep_latin
            ));
        }
    }
    index.push_str("</body></html>");
    write!(file, "{}", index)?;

    Ok(())
}
