use anyhow::anyhow;
use pekzep_syllable::PekZepSyllable;
use serde_derive::Deserialize as De;
use std::fs::File;

use super::char_pronunciation::LinzklarString;

#[derive(Debug, De)]
struct Record {
    characters: String,
    sound: String,
}

/// a lookup table from a sequence of linzklars to a contracted syllable
pub type SoundTable = Vec<(LinzklarString, PekZepSyllable)>;

#[allow(clippy::tabs_in_doc_comments)]
/// Parses "raw/contraction.tsv" to obtain a table converting a string of characters to a contracted syllable.
/// The tsv used for the input should be of the following form:
/// ```text
///characters	sound
///足手	xiop1
/// ```
/// Each of the first column must be a sequence of linzklars. Each of the second column must be a valid Pekzep syllable.
/// # Errors
/// Gives errors if:
/// - IO fails
/// - "raw/contraction.tsv" does not conform to an expected format
/// - the Pekzep is unparsable
///
pub fn parse() -> anyhow::Result<SoundTable> {
    fn convert(record: &Record) -> anyhow::Result<(LinzklarString, PekZepSyllable)> {
        match PekZepSyllable::parse(&record.sound) {
            None => Err(anyhow!("Invalid sound {}", record.sound)),
            Some(a) => Ok((LinzklarString::new(&record.characters)?, a)),
        }
    }

    let f = File::open("raw/contraction.tsv")?;
    let mut rdr = csv::ReaderBuilder::new().delimiter(b'\t').from_reader(f);
    let mut ans = vec![];
    for result in rdr.deserialize() {
        let record: Record = result?;
        ans.push(record);
    }

    ans.iter().map(convert).collect::<anyhow::Result<_>>()
}
