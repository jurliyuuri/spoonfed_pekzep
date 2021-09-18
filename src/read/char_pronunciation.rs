use anyhow::anyhow;
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

impl std::fmt::Display for Linzklar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Linzklar(char);
impl Linzklar {
    fn new(a: &str) -> anyhow::Result<Self> {
        let c = a.chars().next().ok_or_else(|| anyhow!("Got an empty string"))?;
        if a.chars().count() > 1 {
            return Err(anyhow!(
                "Got `{}`, a string longer than a single character",
                a
            ));
        }
        match c {
            '\u{3400}'..='\u{4DBF}' | '\u{4E00}'..='\u{9FFF}' => Ok(Self(c)),
            _ => Err(anyhow!(
                "`{}` is not a character in the Unicode block \"CJK Unified Ideographs\" nor a character in the Unicode block \"CJK Unified Ideographs Extension A\"",
                c
            )),
        }
    }
}
pub type CharSoundTable = Vec<(Linzklar, PekZepSyllable)>;
pub type NonRecommendedCharTable = HashMap<Linzklar, Linzklar>;

#[allow(clippy::tabs_in_doc_comments)]
/// Parses the tsv specified by `path` to obtain a table converting a character to a syllable,
/// as well as a table converting a non-recommended character into a recommended alternative.
/// The tsv used for the input should be of the following form:
/// ```text
///character	sound	variant_of
///之	a	
///噫	a	
///吁	a	噫
///四	ap1	
///御	am	
///禦	am	御
/// ```
/// Each of the first column must be a linzklar. Each of the second column must be a valid Pekzep syllable. The third column must either be 
/// # Errors
/// Gives errors if:
/// - IO fails
/// - the file specified by the `path` does not conform to an expected format
/// - the Pekzep is unparsable
///
pub fn parse(path: &str) -> anyhow::Result<(CharSoundTable, NonRecommendedCharTable)> {
    fn convert(record: &Record) -> anyhow::Result<(Linzklar, PekZepSyllable)> {
        match PekZepSyllable::parse(&record.sound) {
            None => Err(anyhow!("Invalid sound {}", record.sound)),
            Some(a) => Ok((Linzklar::new(&record.character)?, a)),
        }
    }

    let f = File::open(path)?;
    let mut rdr = csv::ReaderBuilder::new().delimiter(b'\t').from_reader(f);
    let mut ans = vec![];
    for result in rdr.deserialize() {
        let record: Record = result?;
        ans.push(record);
    }

    let a = ans.iter().map(convert).collect::<anyhow::Result<_>>()?;

    let mut b = HashMap::new();
    for r in ans {
        if r.variant_of.is_empty() {
        } else {
            b.insert(Linzklar::new(&r.character)?, Linzklar::new(&r.variant_of)?);
        }
    }

    Ok((a, b))
}
