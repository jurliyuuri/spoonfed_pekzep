#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::non_ascii_literal)]
#[macro_use]
extern crate lazy_static;
use askama::Template;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
mod filters;
mod read;
pub mod verify;

#[cfg(test)]
mod tests {
    #[test]
    fn test_split_at_slashslash() {
        use crate::split_at_slashslash;
        assert_eq!(split_at_slashslash("行 // 道"), (String::from("行 "), String::from(" 道")))
    }
}

/// Splits the string at the first occurrence of `//`.
/// # Panic
/// Panics if the string does not have a `//`.
fn split_at_slashslash(in_string: &str) -> (String, String) {
    let mut splitter = in_string.splitn(2, "//");
    let first = splitter.next().unwrap();
    let second = splitter.next().unwrap();
    (first.to_owned(), second.to_owned())
}

impl read::vocab::Item {
    pub fn to_tab_separated(&self, rel_path: &'static str) -> String {
        self.to_tab_separated_with_custom_linzifier(|s| {
            convert_hanzi_to_images(s, "/{} N()SL", rel_path)
        })
    }
}
impl verify::DecompositionItem {
    #[must_use]
    pub fn to_tab_separated_with_splittable_compound_info_and_also_with_a_link(
        &self,
        rel_path: &'static str,
    ) -> String {
        let link_path = format!("{}/vocab/{}.html", rel_path, self.key.replace(" // ", "_slashslash_"));
        if let Some(splittable) = self.splittable_compound_info {
            let (latin_former, latin_latter) = split_at_slashslash(&self.voc.pekzep_latin);
            let (hanzi_former, hanzi_latter) = split_at_slashslash(&self.voc.pekzep_hanzi);
            match splittable {
                verify::SplittableCompoundInfo::FormerHalfHash => {
                    format!(
                        "<a href=\"{}\">{}<span style=\"font-size: 75%; color: #444\">//{}</span></a>\t{}<span style=\"font-size: 75%; color: #444\">//{}</span>\t<span style=\"filter:brightness(65%)contrast(500%);\">{}</span>//<span style=\"filter:brightness(80%)contrast(80%);\">{}</span>\t{}\t{}\t{}",
                        link_path,
                        latin_former, 
                        latin_latter,
                        hanzi_former, 
                        hanzi_latter,
                        &convert_hanzi_to_images_with_size(&hanzi_former, "/{} N()SL", rel_path, 30), 
                        &convert_hanzi_to_images_with_size(&hanzi_latter, "/{} N()SL", rel_path, 22), 
                        self.voc.parts_of_speech,
                        self.voc.parts_of_speech_supplement,
                        self.voc.english_gloss
                    )
                }
                verify::SplittableCompoundInfo::LatterHalfExclamation => {
                    format!(
                        "<a href=\"{}\"><span style=\"font-size: 75%; color: #444\">{}//</span>{}</a>\t<span style=\"font-size: 75%; color: #444\">{}//</span>{}\t<span style=\"filter:brightness(80%)contrast(80%);\">{}</span>//<span style=\"filter:brightness(65%)contrast(500%);\">{}</span>\t{}\t{}\t{}",
                        link_path,
                        latin_former, 
                        latin_latter,
                        hanzi_former, 
                        hanzi_latter,
                        &convert_hanzi_to_images_with_size(&hanzi_former, "/{} N()SL", rel_path, 22), 
                        &convert_hanzi_to_images_with_size(&hanzi_latter, "/{} N()SL", rel_path, 30), 
                        self.voc.parts_of_speech,
                        self.voc.parts_of_speech_supplement,
                        self.voc.english_gloss
                    )
                }
            }
        } else {
            format!(
                "<a href=\"{}\">{}</a>\t{}\t<span style=\"filter:brightness(65%)contrast(500%);\">{}</span>\t{}\t{}\t{}",
                link_path,
                self.voc.pekzep_latin,
                self.voc.pekzep_hanzi,
                convert_hanzi_to_images(&self.voc.pekzep_hanzi,  "/{} N()SL", rel_path),
                self.voc.parts_of_speech,
                self.voc.parts_of_speech_supplement,
                self.voc.english_gloss
            )
        }
    }
}

#[derive(Template)]
#[template(path = "phrase.html")]
struct PhraseTemplate<'a> {
    english: &'a str,
    chinese_hanzi: &'a str,
    chinese_pinyin: &'a str,
    pekzep_latin: &'a str,
    pekzep_hanzi: &'a str,
    prev_link: &'a str,
    next_link: &'a str,
    wav_tag: &'a str,
    analysis: &'a str,
    oga_tag: &'a str,
    pekzep_images: &'a str,
    author_color: &'a str,
    author_name: &'a str,
    has_audio: bool,
}

#[derive(Template)]
#[template(path = "ind.html")]
struct IndTemplate<'a> {
    index: &'a str,
    length: usize,
    how_many_glosses: usize,
}

#[derive(Template)]
#[template(path = "vocab.html")]
struct VocabTemplate<'a> {
    analysis: &'a str,
    usage_table: &'a str,
}

#[derive(Template)]
#[template(path = "vocab_list.html")]
struct VocabListTemplate<'a> {
    vocab_html: &'a str,
}

const fn to_check(a: bool) -> &'static str {
    if a {
        "&#x2713;"
    } else {
        ""
    }
}

fn char_img_with_size(name: &str, rel_path: &'static str, size: usize) -> String {
    use log::info;
    if std::path::Path::new(&format!("raw/char_img/{}.png", name)).exists() {
        // only copy the files that are actually used
        match std::fs::copy(
            format!("raw/char_img/{}.png", name),
            format!("docs/char_img/{}.png", name),
        ) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Copying raw/char_img/{}.png failed: {}", name, e);
            }
        }
    } else {
        info!("char_img not found: {}.png", name);
        File::create(&format!("docs/char_img/dummy_{}.txt", name)).unwrap();
    }

    format!(
        r#"<img src="{}/char_img/{}.png" height="{}">"#,
        rel_path, name, size
    )
}

fn convert_hanzi_to_images(s: &str, exclude_list: &str, rel_path: &'static str) -> String {
    convert_hanzi_to_images_with_size(s, exclude_list, rel_path, 30)
}

fn convert_hanzi_to_images_with_size(s: &str, exclude_list: &str, rel_path: &'static str, size: usize) -> String {
    let mut ans = String::new();
    let mut iter = s.chars();
    let mut remove_following_space = false;
    while let Some(c) = iter.next() {
        if c == '∅' {
            ans.push_str(&char_img_with_size("blank", rel_path, size))
        } else if c == 'x' {
            if Some('i') == iter.next() && Some('z') == iter.next() && Some('i') == iter.next() {
                ans.push_str(&char_img_with_size("xi", rel_path, size));
                ans.push_str(&char_img_with_size("zi", rel_path, size));
                remove_following_space = true; // this deletes the redundant space after "xizi"
            } else {
                panic!("Expected `xizi` because `x` was encountered, but did not find it.")
            }
        } else if exclude_list.contains(c) {
            if !(remove_following_space && c == ' ') {
                ans.push(c);
            }
        } else {
            ans.push_str(&char_img_with_size(&c.to_string(), rel_path, size))
        }
    }

    ans
}

fn generate_oga_tag(row: &read::phrase::Item, syllables: &[read::phrase::ExtSyllable]) -> String {
    use log::warn;
    let filename = read::phrase::syllables_to_str_underscore(syllables);
    if row.filetype.contains(&read::phrase::FilePathType::Oga) {
        if !std::path::Path::new(&format!("docs/spoonfed_pekzep_sounds/{}.oga", filename)).exists()
        {
            warn!("oga file not found: {}.oga", filename)
        }
        format!(
            r#"<source src="../spoonfed_pekzep_sounds/{}.oga" type="audio/ogg">"#,
            filename
        )
    } else {
        if std::path::Path::new(&format!("docs/spoonfed_pekzep_sounds/{}.oga", filename)).exists() {
            warn!("oga file IS found, but is not linked: {}.oga", filename)
        }
        "".to_owned()
    }
}

fn generate_wav_tag(row: &read::phrase::Item, syllables: &[read::phrase::ExtSyllable]) -> String {
    use log::warn;
    if row.filetype.contains(&read::phrase::FilePathType::Wav)
        || row.filetype.contains(&read::phrase::FilePathType::WavR)
    {
        let filename = if row.filetype.contains(&read::phrase::FilePathType::WavR) {
            read::phrase::syllables_to_rerrliratixka_no_space(syllables)
        } else {
            read::phrase::syllables_to_str_underscore(syllables)
        };

        if !std::path::Path::new(&format!("docs/spoonfed_pekzep_sounds/{}.wav", filename)).exists()
        {
            warn!("wav file not found: {}.wav", filename)
        }
        format!(
            r#"<source src="../spoonfed_pekzep_sounds/{}.wav" type="audio/wav">"#,
            filename
        )
    } else {
        let filename = read::phrase::syllables_to_rerrliratixka_no_space(syllables);
        if std::path::Path::new(&format!("docs/spoonfed_pekzep_sounds/{}.wav", filename)).exists() {
            warn!("wav file IS found, but is not linked: {}.wav", filename)
        }

        let filename = read::phrase::syllables_to_str_underscore(syllables);
        if std::path::Path::new(&format!("docs/spoonfed_pekzep_sounds/{}.wav", filename)).exists() {
            warn!("wav file IS found, but is not linked: {}.wav", filename)
        }
        "".to_owned()
    }
}

/// Generates `phrase/`
/// # Errors
/// Will return `Err` if the file I/O fails or the render panics.
pub fn generate_phrases(data_bundle: &verify::DataBundle) -> Result<(), Box<dyn Error>> {
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
        let analysis = decomposition
            .iter()
            .map(|decomposition_item| {
                decomposition_item.to_tab_separated_with_splittable_compound_info_and_also_with_a_link("..")
            })
            .collect::<Vec<_>>();

        let pekzep_hanzi_guillemet_removed = row.pekzep_hanzi.replace("«", "").replace("»", "");

        let content = PhraseTemplate {
            english: &row.english,
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
            oga_tag: &generate_oga_tag(row, syllables),
            analysis: &analysis.join("\n"),
            pekzep_images: &convert_hanzi_to_images(&pekzep_hanzi_guillemet_removed, "() ", ".."),
            author_color: match &row.recording_author {
                Some(read::phrase::Author::JektoVatimeliju) => "#754eab",
                Some(read::phrase::Author::FaliraLyjotafis) => "#e33102",
                Some(s) => {
                    warn!("Unrecognized author `{:?}`", s);
                    "#000000"
                }
                None => "#000000",
            },
            author_name: &match &row.recording_author {
                Some(author) => format!("{}", author),
                None => "".to_string(),
            },
            has_audio: row.recording_author.is_some(),
        };
        write!(file, "{}", content.render()?)?;

        if row.chinese_hanzi.starts_with('A') && row.chinese_hanzi.contains('B') {
            warn!("A-B style dialogue detected: {}, matched with {}. Replace this with 「」-style while also making sure the Hanzi and the Pinyin matches.", row.chinese_hanzi, row.chinese_pinyin)
        }
    }
    Ok(())
}

/// Generates `vocab/`
/// # Errors
/// Will return `Err` if the file I/O fails or the render panics.
pub fn generate_vocabs(data_bundle: &verify::DataBundle) -> Result<(), Box<dyn Error>> {
    for (key, v) in &data_bundle.vocab_ordered {
        let mut file = File::create(format!(
            "docs/vocab/{}.html",
            key.replace(" // ", "_slashslash_")
        ))?;

        let mut usages = String::from("");

        for verify::Rows3Item {syllables, decomposition,  row} in &data_bundle.rows3 {
            if decomposition.iter().any(|item| &item.key == key) {
                usages += &format!(r#"
            <div style="margin-left: 10px; border-left: 3px solid rgb(34,126,188); padding-left: 5px">
                <p><span lang="ja">{}</span></p>
                <p><a href="../phrase/{}.html">{}</a></p>
                <p><span lang="en">{}</span> / <span lang="zh-CN">{}</span></p>
            </div>"#, 
            row.pekzep_hanzi,
            read::phrase::syllables_to_str_underscore(syllables), row.pekzep_latin,
            
            row.english,  row.chinese_hanzi)
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

/// Generates `vocab_list_internal.html`
/// # Errors
/// Will return `Err` if the file I/O fails or the render panics.
pub fn generate_vocab_list_internal(data_bundle: &verify::DataBundle) -> Result<(), Box<dyn Error>> {
    let mut vocab_file = File::create("docs/vocab_list_internal.html")?;
    let mut vocab_html = vec![];
    for (key, vocab) in &data_bundle.vocab_ordered {
        vocab_html.push(format!("{}\t{}", key, vocab.to_tab_separated(".")))
    }
    write!(
        vocab_file,
        "{}",
        VocabListTemplate {
            vocab_html: &vocab_html.join("\n")
        }
        .render()?
    )?;
    Ok(())
}

/// Generates `vocab_list.html`
/// # Errors
/// Will return `Err` if the file I/O fails or the render panics.
pub fn generate_vocab_list(data_bundle: &verify::DataBundle) -> Result<(), Box<dyn Error>> {
    let mut vocab_file = File::create("docs/vocab_list.html")?;
    let mut vocab_html = vec![];
    for (_, vocab) in &data_bundle.vocab_ordered {
        vocab_html.push(vocab.to_tab_separated("."))
    }
    write!(
        vocab_file,
        "{}",
        VocabListTemplate {
            vocab_html: &vocab_html.join("\n")
        }
        .render()?
    )?;
    Ok(())
}

/// Generates `index.html`
/// # Errors
/// Will return `Err` if the file I/O fails or the render panics.
pub fn generate_index(data_bundle: &verify::DataBundle) -> Result<(), Box<dyn Error>> {
    let mut file = File::create("docs/index.html")?;
    let mut index = vec!["<abbr title=\"Audio available in Edge, Firefox, Chrome and Opera. / 在Edge、Firefox、Chrome和Opera中都可以听到录音。\">🔊<i class=\"fab fa-chrome\"></i><i class=\"fab fa-firefox-browser\"></i><i class=\"fab fa-edge\"></i><i class=\"fab fa-edge-legacy\"></i><i class=\"fab fa-opera\"></i></abbr>\t<abbr title=\"Audio available in Safari. / 在Safari中都可以听到录音。\">🔊<i class=\"fab fa-safari\"></i></abbr>\tanalysis\tphrase".to_string()];
    let mut how_many_glosses = 0;
    for verify::Rows3Item {
        syllables,
        decomposition,
        row,
    } in &data_bundle.rows3
    {
        if !decomposition.is_empty() {
            how_many_glosses += 1;
        }

        index.push(format!(
            "{}\t{}\t{}\t<a href=\"phrase/{}.html\">{}</a>",
            to_check(
                row.filetype.contains(&read::phrase::FilePathType::Wav)
                    || row.filetype.contains(&read::phrase::FilePathType::WavR)
                    || row.filetype.contains(&read::phrase::FilePathType::Oga)
            ),
            to_check(
                row.filetype.contains(&read::phrase::FilePathType::Wav)
                    || row.filetype.contains(&read::phrase::FilePathType::WavR)
            ),
            to_check(!decomposition.is_empty()),
            read::phrase::syllables_to_str_underscore(syllables),
            row.pekzep_latin
        ));
    }

    write!(
        file,
        "{}",
        IndTemplate {
            index: &index.join("\n"),
            length: index.len() - 1, /* subtract off the title row */
            how_many_glosses
        }
        .render()?
    )?;

    Ok(())
}

/// Generates `raw.tsv`
/// # Errors
/// Will return `Err` if the file I/O fails or the render panics.
pub fn write_condensed_csv() -> Result<(), Box<dyn Error>> {
    use csv::StringRecord;
    use filters::normalizer::{
        capitalize_first_char, normalize_a_b_dialogue, normalize_chinese_punctuation,
    };
    use read::phrase::Record;
    use std::io::BufReader;
    let f = File::open("raw/Spoonfed Pekzep - SpoonfedPekzep.tsv")?;
    let f = BufReader::new(f);
    let mut condensed_csv = String::new();
    for line in f.lines() {
        // to prevent double quotes from vanishing, I do not read with CSV parser
        let rec: Record =
            StringRecord::from(line?.split('\t').collect::<Vec<_>>()).deserialize(None)?;

        // 未査読の行は飛ばす
        if rec.pekzep_hanzi.contains('@') {
            continue;
        }
        if rec.pekzep_latin.is_empty() {
            continue;
        }

        if rec.requires_substitution.is_empty() {
            condensed_csv += &format!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                rec.english,
                rec.pekzep_latin,
                rec.pekzep_hanzi,
                capitalize_first_char(&rec.chinese_pinyin),
                normalize_a_b_dialogue(&normalize_chinese_punctuation(&rec.chinese_hanzi)),
                rec.decomposed,
                rec.filetype,
                rec.recording_author,
            )
        }
    }

    std::fs::write("docs/raw.tsv", condensed_csv)?;
    Ok(())
}



/// Generates `raw.js`
/// # Errors
/// Will return `Err` if the file I/O fails or the render panics.
pub fn write_condensed_js() -> Result<(), Box<dyn Error>> {
    use csv::StringRecord;
    use filters::normalizer::{
        capitalize_first_char, normalize_a_b_dialogue, normalize_chinese_punctuation,
    };
    use read::phrase::Record;
    use std::io::BufReader;
    let f = File::open("raw/Spoonfed Pekzep - SpoonfedPekzep.tsv")?;
    let f = BufReader::new(f);
    let mut js = String::from("const RAW_DATA = [\n");
    for line in f.lines() {
        // to prevent double quotes from vanishing, I do not read with CSV parser
        let rec: Record =
            StringRecord::from(line?.split('\t').collect::<Vec<_>>()).deserialize(None)?;

        // 未査読の行は飛ばす
        if rec.pekzep_hanzi.contains('@') {
            continue;
        }
        if rec.pekzep_latin.is_empty() {
            continue;
        }

        if rec.requires_substitution.is_empty() {

            // This is inherently insecure, but who cares?
            js += &format!(
                "\t{{english: `{}`, pekzep_latin: `{}`, pekzep_hanzi: `{}`, chinese_pinyin: `{}`, chinese_hanzi: `{}`, decomposed: `{}`, filetype: `{}`, recording_author: `{}`}},\n",
                rec.english,
                rec.pekzep_latin,
                rec.pekzep_hanzi,
                capitalize_first_char(&rec.chinese_pinyin),
                normalize_a_b_dialogue(&normalize_chinese_punctuation(&rec.chinese_hanzi)),
                rec.decomposed,
                rec.filetype,
                rec.recording_author,
            )
        }
    }

    js += "]\n";

    std::fs::write("docs/raw.js", js)?;
    Ok(())
}