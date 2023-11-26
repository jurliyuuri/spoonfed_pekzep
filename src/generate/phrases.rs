use crate::generate::audio_tag::{generate_oga_tag, generate_wav_tag};
use askama::Template;

use crate::askama_templates::PhraseTemplate;
use crate::read;
use crate::{
    convert_hanzi_to_images, remove_guillemets,
    sentence_decomposition_to_analysis_merging_unsplitted_compounds, verify,
};
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;

/// Generates `phrase/`
/// # Errors
/// Will return `Err` if the file I/O fails or the render panics.
pub fn gen(data_bundle: &verify::DataBundle) -> Result<(), Box<dyn Error>> {
    use log::warn;
    for (
        i,
        verify::Rows3Item {
            syllables,
            decomposition,
            row,
        },
    ) in data_bundle.rows3.iter().enumerate()
    {
        let prev = if i == 0 {
            None
        } else {
            data_bundle.rows3.get(i - 1)
        };
        let next = data_bundle.rows3.get(i + 1);
        if row.pekzep_latin.is_empty() {
            continue;
        }
        let mut file = File::create(format!(
            "docs/phrase/{}.html",
            read::phrase::syllables_to_str_underscore(syllables)
        ))?;

        let pekzep_hanzi_guillemet_removed = remove_guillemets(&row.pekzep_hanzi);
        let (oga_tag, is_reviewed) = generate_oga_tag(row, syllables);
        let content = PhraseTemplate {
            english: &row.english,
            japanese: &row.japanese,
            has_japanese: row.japanese.trim() != "",
            chinese_pinyin: &row.chinese_pinyin,
            chinese_hanzi: &row.chinese_hanzi,
            pekzep_latin: &row.pekzep_latin,
            pekzep_hanzi: &pekzep_hanzi_guillemet_removed,
            prev_link: &match prev {
                None => "../index".to_string(),
                Some(verify::Rows3Item { syllables, .. }) => {
                    read::phrase::syllables_to_str_underscore(syllables)
                }
            },
            next_link: &match next {
                None => "../index".to_string(),
                Some(verify::Rows3Item { syllables, .. }) => {
                    read::phrase::syllables_to_str_underscore(syllables)
                }
            },
            wav_tag: &generate_wav_tag(row, syllables),
            oga_tag: &oga_tag,
            analysis: &decomposition
                .iter()
                .map(|sentence| {
                    sentence_decomposition_to_analysis_merging_unsplitted_compounds(sentence)
                        .join("\n")
                })
                .collect::<Vec<_>>()
                .join("\n\n"),
            pekzep_images: &convert_hanzi_to_images(&pekzep_hanzi_guillemet_removed, "() ", ".."),
            author_color: match (&row.recording_author, is_reviewed) {
                (_, Some(false)) => "#ff00ff",
                (Some(read::phrase::Author::JektoVatimeliju), _) => "#754eab",
                (Some(read::phrase::Author::FaliraLyjotafis), _) => "#e33102",
                (Some(s), _) => {
                    warn!("Unrecognized author `{:?}`", s);
                    "#000000"
                }
                (None, _) => "#000000",
            },
            author_name: &if is_reviewed == Some(false) {
                "jekto.vatimeliju (not reviewed)".to_string()
            } else {
                row.recording_author
                    .as_ref()
                    .map_or_else(String::new, |author| format!("{author}"))
            },
            has_audio: row.recording_author.is_some() || is_reviewed == Some(false),
        };
        write!(file, "{}", content.render()?)?;

        if row.chinese_hanzi.starts_with('A') && row.chinese_hanzi.contains('B') {
            warn!("A-B style dialogue detected: {}, matched with {}. Replace this with 「」-style while also making sure the Hanzi and the Pinyin matches.", row.chinese_hanzi, row.chinese_pinyin);
        }
    }
    Ok(())
}
