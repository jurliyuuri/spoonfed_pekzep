use askama::Template;

use crate::askama_templates::VocabTemplate;
use crate::read;
use crate::verify;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;

/// Generates `vocab/`
/// # Errors
/// Will return `Err` if the file I/O fails or the render panics.
pub fn r#gen(data_bundle: &verify::DataBundle) -> Result<(), Box<dyn Error>> {
    for (key, v) in &data_bundle.vocab_ordered {
        let mut file = File::create(format!("docs/vocab/{}.html", key.to_path_safe_string()))?;

        let mut usages = String::new();

        for verify::Rows3Item {
            syllables,
            decomposition,
            row,
        } in &data_bundle.rows3
        {
            if decomposition.iter().flatten().any(|item| item.key == *key) {
                usages += &format!(
                    r#"
            <div style="margin-left: 10px; border-left: 3px solid rgb(34,126,188); padding-left: 5px">
                <p><span lang="ja">{}</span></p>
                <p><a href="../phrase/{}.html">{}</a></p>
                <p><span lang="en">{}</span> / <span lang="zh-CN">{}</span></p>
            </div>"#,
                    row.pekzep_hanzi,
                    read::phrase::syllables_to_str_underscore(syllables),
                    row.pekzep_latin,
                    row.english,
                    row.chinese_hanzi
                );
            }
        }

        write!(
            file,
            "{}",
            VocabTemplate {
                analysis: &v.to_tab_separated(".."),
                usage_table: &usages
            }
            .render()?
        )?;
    }
    Ok(())
}
