use anyhow::anyhow;
use partition_eithers::collect_any_errors;
use pekzep_syllable::PekZepSyllable;
use serde_derive::Deserialize as De;
use std::collections::HashMap;
use std::fs::File;

#[derive(Debug, De)]
struct Record {
    character: String,
    sound: String,
    variant_of: String,
}

pub type CharSoundTable = Vec<(String, PekZepSyllable)>;
pub type NonRecommendedCharTable = HashMap<String, String>;

/// Parses the tsv specified by `path` to obtain a table converting a character to a syllable, 
/// as well as a table converting a non-recommended character into a recommended alternative.
/// # Errors
/// Gives errors if:
/// - IO fails
/// - the file specified by the `path` does not conform to an expected format
/// - the Pekzep is unparsable
///
pub fn parse(path: &str) -> anyhow::Result<(CharSoundTable, NonRecommendedCharTable)> {
    fn convert(record: &Record) -> Result<(String, PekZepSyllable), String> {
        match PekZepSyllable::parse(&record.sound) {
            None => Err(format!("Invalid sound {}", record.sound)),
            Some(a) => Ok((record.character.clone(), a)),
        }
    }

    let f = File::open(path)?;
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
