use anyhow::anyhow;
use askama::Template;

use crate::askama_templates::VocabListInternalTemplate;
use crate::verify;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;

/// Generates `vocab_list_internal.html`
/// # Errors
/// Will return `Err` if the file I/O or the rendering fails.
pub fn r#gen(data_bundle: &verify::DataBundle) -> Result<(), Box<dyn Error>> {
    let mut file = File::create("docs/vocab_list_internal.html")?;
    let mut html = vec![];
    for (key, vocab) in &data_bundle.vocab_ordered {
        let rel_path = ".";
        let link_path = format!("{rel_path}/vocab/{}.html", key.to_path_safe_string());
        html.push(format!(
            "<a href=\"{}\">{}</a>\t{}\t{}",
            link_path,
            key,
            data_bundle
                .vocab_count
                .get(key)
                .ok_or_else(|| anyhow!("vocab_count should be consistent with vocab_ordered"))?,
            vocab.to_tab_separated(rel_path)
        ));
    }
    write!(
        file,
        "{}",
        VocabListInternalTemplate {
            vocab_html: &html.join("\n"),
            header_row: "internal word id\toccurrence\tPekzep (alphabet)\tPekzep (Chinese characters)\tPekzep (Linzklā)\tparts of speech\tsubdivision\tEnglish translation"
        }
        .render()?
    )?;
    Ok(())
}
