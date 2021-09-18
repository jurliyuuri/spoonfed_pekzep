use anyhow::anyhow;
use partition_eithers::collect_any_errors;
use pekzep_syllable::PekZepSyllable;
use serde_derive::Deserialize as De;
use std::fs::File;

#[derive(Debug, De)]
struct Record {
    characters: String,
    sound: String,
}

pub type SoundTable = Vec<(String, PekZepSyllable)>;

pub fn parse() -> anyhow::Result<SoundTable> {
    fn convert(record: &Record) -> Result<(String, PekZepSyllable), String> {
        match PekZepSyllable::parse(&record.sound) {
            None => Err(format!("Invalid sound {}", record.sound)),
            Some(a) => Ok((record.characters.clone(), a)),
        }
    }

    let f = File::open("raw/contraction.tsv")?;
    let mut rdr = csv::ReaderBuilder::new().delimiter(b'\t').from_reader(f);
    let mut ans = vec![];
    for result in rdr.deserialize() {
        let record: Record = result?;
        ans.push(record);
    }

    let a: anyhow::Result<Vec<(String, PekZepSyllable)>> =
        collect_any_errors(ans.iter().map(convert).collect::<Vec<_>>())
            .map_err(|e| anyhow!(e.join("\n")));

    let a: Vec<(String, PekZepSyllable)> = a?;

    Ok(a)
}
