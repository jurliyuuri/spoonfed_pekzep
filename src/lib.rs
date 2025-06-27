#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::non_ascii_literal, clippy::too_many_lines)]
use askama::Template;
use read::char_pronunciation::Linzklar;

use crate::askama_templates::{CharListTemplate, IndTemplate, VocabListTemplate};
use crate::read::vocab::SplittableCompoundInfo;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Write as _;
use std::fs::File;
use std::io::prelude::*;

/// reads from the input files
pub mod read;

/// checks whether all the data collected from the input files are consistent with each other
pub mod verify;

/// Pure functions that are used to normalize the input
pub mod normalizer;

/// used by [askama](https://djc.github.io/askama/) to generate HTML
pub mod askama_templates;

pub mod generate;

mod recurse;

/// Splits the string at the first occurrence of `//`.
/// # Panic
/// Panics if the string does not have a `//`.
fn split_at_slashslash(in_string: &str) -> (String, String) {
    let (first, second) = in_string.split_once("//").unwrap();
    (first.to_owned(), second.to_owned())
}

impl read::vocab::Item {
    #[must_use]
    pub fn to_tab_separated(&self, rel_path: &'static str) -> String {
        self.to_tab_separated_with_custom_linzifier(|s| {
            convert_hanzi_to_images(s, "/{} N()SL«»", rel_path)
        })
    }
}
impl verify::DecompositionItem {
    #[must_use]
    pub fn to_tab_separated_with_splittable_compound_info_and_also_with_a_link(
        &self,
        rel_path: &'static str,
    ) -> String {
        let link_path = format!("{rel_path}/vocab/{}.html", self.key.to_path_safe_string());
        self.splittable_compound_info.map_or_else(|| format!(
                "<a href=\"{}\">{}</a>\t{}\t<span style=\"filter:brightness(65%) contrast(500%);\">{}</span>\t{}\t{}\t{}",
                link_path,
                self.voc.pekzep_latin,
                self.voc.pekzep_hanzi,
                convert_hanzi_to_images(&self.voc.pekzep_hanzi,  "/{} N()SL«»", rel_path),
                self.voc.parts_of_speech,
                self.voc.parts_of_speech_supplement,
                self.voc.english_gloss
            ), |splittable| {
            let (latin_former, latin_latter) = split_at_slashslash(&self.voc.pekzep_latin);
            let (hanzi_former, hanzi_latter) = split_at_slashslash(&self.voc.pekzep_hanzi);
            match splittable {
                SplittableCompoundInfo::FormerHalfHash => {
                    format!(
                        "<a href=\"{}\">{}<span style=\"font-size: 75%; color: #444\">//{}</span></a>\t{}<span style=\"font-size: 75%; color: #444\">//{}</span>\t<span style=\"filter:brightness(65%) contrast(500%);\">{}</span>//<span style=\"filter:brightness(80%) contrast(80%);\">{}</span>\t{}\t{}\t{}",
                        link_path,
                        latin_former,
                        latin_latter,
                        hanzi_former,
                        hanzi_latter,
                        &convert_hanzi_to_images_with_size(&hanzi_former, "/{} N()SL«»", rel_path, 30),
                        &convert_hanzi_to_images_with_size(&hanzi_latter, "/{} N()SL«»", rel_path, 22),
                        self.voc.parts_of_speech,
                        self.voc.parts_of_speech_supplement,
                        self.voc.english_gloss
                    )
                }
                SplittableCompoundInfo::LatterHalfExclamation => {
                    format!(
                        "<a href=\"{}\"><span style=\"font-size: 75%; color: #444\">{}//</span>{}</a>\t<span style=\"font-size: 75%; color: #444\">{}//</span>{}\t<span style=\"filter:brightness(80%) contrast(80%);\">{}</span>//<span style=\"filter:brightness(65%) contrast(500%);\">{}</span>\t{}\t{}\t{}",
                        link_path,
                        latin_former,
                        latin_latter,
                        hanzi_former,
                        hanzi_latter,
                        &convert_hanzi_to_images_with_size(&hanzi_former, "/{} N()SL«»", rel_path, 22),
                        &convert_hanzi_to_images_with_size(&hanzi_latter, "/{} N()SL«»", rel_path, 30),
                        self.voc.parts_of_speech,
                        self.voc.parts_of_speech_supplement,
                        self.voc.english_gloss
                    )
                }
            }
        })
    }
}

const fn to_check(a: bool) -> &'static str {
    if a { "&#x2713;" } else { "" }
}

#[derive(Copy, Clone, Debug)]
enum R {
    Ready,
    NonReviewed,
    Missing,
}

const fn to_check_or_parencheck(a: R) -> &'static str {
    match a {
        R::Ready => "&#x2713;",
        R::NonReviewed => "(&#x2713;)",
        R::Missing => "",
    }
}

fn char_img_with_size(name: &str, rel_path: &'static str, size: usize, gen_link: bool) -> String {
    use log::info;
    if std::path::Path::new(&format!("raw/char_img/{name}.png")).exists() {
        // only copy the files that are actually used
        match std::fs::copy(
            format!("raw/char_img/{name}.png"),
            format!("docs/char_img/{name}.png"),
        ) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Copying raw/char_img/{name}.png failed: {e}");
            }
        }
    } else if std::path::Path::new(&format!("raw/char_img_fallback/{name}.png")).exists() {
        match std::fs::copy(
            format!("raw/char_img_fallback/{name}.png"),
            format!("docs/char_img/{name}.png"),
        ) {
            Ok(_) => {
                info!(
                    "char_img not found, but found in char_img_fallback: {}.png",
                    name
                );
                File::create(format!("docs/char_img/fallback_{name}.txt")).unwrap();
            }
            Err(e) => {
                eprintln!("Copying raw/char_img_fallback/{name}.png failed: {e}");
            }
        }
    } else {
        info!("char_img not found: {}.png", name);
        File::create(format!("docs/char_img/dummy_{name}.txt")).unwrap();
    }
    if gen_link {
        format!(
            r#"<a href="{rel_path}/char/{name}.html"><img src="{rel_path}/char_img/{name}.png" height="{size}"></a>"#,
        )
    } else {
        format!(r#"<img src="{rel_path}/char_img/{name}.png" height="{size}">"#,)
    }
}

fn convert_hanzi_to_images(s: &str, exclude_list: &str, rel_path: &'static str) -> String {
    convert_hanzi_to_images_with_size(s, exclude_list, rel_path, 30)
}

fn convert_hanzi_to_images_with_size(
    s: &str,
    exclude_list: &str,
    rel_path: &'static str,
    size: usize,
) -> String {
    let mut ans = String::new();
    let mut iter = s.chars();
    let mut remove_following_space = false;
    while let Some(c) = iter.next() {
        if c == '∅' {
            ans.push_str(&char_img_with_size("blank", rel_path, size, false));
        } else if c == 'x' {
            if Some('i') == iter.next() && Some('z') == iter.next() && Some('i') == iter.next() {
                ans.push_str(&char_img_with_size("xi", rel_path, size, false));
                ans.push_str(&char_img_with_size("zi", rel_path, size, false));
                remove_following_space = true; // this deletes the redundant space after "xizi"
            } else {
                panic!("Expected `xizi` because `x` was encountered, but did not find it.");
            }
        } else if exclude_list.contains(c) {
            if !(remove_following_space && c == ' ') {
                ans.push(c);
            }
        } else {
            if c.is_ascii() {
                log::warn!("Unexpected ASCII character `{}` in {}", c, s);
            }
            ans.push_str(&char_img_with_size(
                &c.to_string(),
                rel_path,
                size,
                Linzklar::is_suitable_charcode_for_linzklar(c),
            ));
        }
    }

    ans
}

fn sentence_decomposition_to_analysis_merging_unsplitted_compounds(
    sentence_decomposition: &[verify::DecompositionItem],
) -> Vec<String> {
    // When splittable compounds appear unsplitted, it is better to display them merged.
    /*
        我
        ∅
        与#学
        与!学
        之
        人
    */
    let mut ans = vec![];

    let mut skip_flag = false;
    for (i, decomposition_item) in sentence_decomposition.iter().enumerate() {
        if Some(SplittableCompoundInfo::FormerHalfHash)
            == decomposition_item.splittable_compound_info
            && Some(SplittableCompoundInfo::LatterHalfExclamation)
                == sentence_decomposition[i + 1].splittable_compound_info
        {
            // Splittable compounds appear unsplitted; it is better to display them merged.
            ans.push(
                verify::DecompositionItem {
                    splittable_compound_info: None,
                    ..(*decomposition_item).clone()
                }
                .to_tab_separated_with_splittable_compound_info_and_also_with_a_link(".."),
            );
            skip_flag = true;
        } else if skip_flag {
            skip_flag = false;
        } else {
            ans.push(
                decomposition_item
                    .to_tab_separated_with_splittable_compound_info_and_also_with_a_link(".."),
            );
        }
    }
    ans
}

fn remove_guillemets(a: &str) -> String {
    a.replace(['«', '»'], "")
}

/// Generates `vocab_list.html`
/// # Errors
/// Will return `Err` if the file I/O fails or the render panics.
pub fn generate_vocab_list(data_bundle: &verify::DataBundle) -> Result<(), Box<dyn Error>> {
    let mut file = File::create("docs/vocab_list.html")?;
    let mut html = vec![];
    for (_, vocab) in &data_bundle.vocab_ordered {
        html.push(vocab.to_tab_separated("."));
    }
    write!(
        file,
        "{}",
        VocabListTemplate {
            vocab_html: &html.join("\n")
        }
        .render()?
    )?;
    Ok(())
}

/// Generates `char_list.html`
/// # Errors
/// Will return `Err` if the file I/O fails or the render panics.
pub fn generate_char_list(data_bundle: &verify::DataBundle) -> Result<(), Box<dyn Error>> {
    let mut file = File::create("docs/char_list.html")?;
    let mut html = vec![];
    let rel_path = ".";

    let mut count_vec: Vec<_> = data_bundle.char_count.iter().collect();
    count_vec.sort_by(|a, b| (b.1, b.0).cmp(&(a.1, a.0)));
    for (linzklar, size) in count_vec {
        html.push(format!(
            "{}\t<span style=\"filter:brightness(65%) contrast(500%);\">{}</span>\t{}",
            linzklar,
            convert_hanzi_to_images(&format!("{linzklar}"), "/{} N()SL«»", rel_path),
            size
        ));
    }
    write!(
        file,
        "{}",
        CharListTemplate {
            char_list_table: &html.join("\n")
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

        let wav_or_oga_is_ready = if row.filetype.contains(&read::phrase::FilePathType::Wav)
            || row.filetype.contains(&read::phrase::FilePathType::Oga)
        {
            R::Ready
        } else {
            let filename = read::phrase::syllables_to_str_underscore(syllables);
            if std::path::Path::new(&format!("docs/nonreviewed_sounds/{filename}.oga")).exists() {
                R::NonReviewed
            } else {
                R::Missing
            }
        };

        index.push(format!(
            "{}\t{}\t{}\t<a href=\"phrase/{}.html\">{}</a>",
            to_check_or_parencheck(wav_or_oga_is_ready),
            to_check(row.filetype.contains(&read::phrase::FilePathType::Wav)),
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
    use log::warn;
    use normalizer::{
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

        // 未査読でもないのに転写が埋まってないやつは警告
        if rec.pekzep_hanzi.is_empty() {
            warn!("The transcription for `{}` is empty.", rec.pekzep_latin);
        }

        if rec.requires_substitution.is_empty() {
            let _ = writeln!(
                condensed_csv,
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                rec.english,
                rec.pekzep_latin,
                rec.pekzep_hanzi,
                capitalize_first_char(&rec.chinese_pinyin),
                normalize_a_b_dialogue(&normalize_chinese_punctuation(&rec.chinese_hanzi)),
                rec.decomposed,
                rec.filetype,
                rec.recording_author,
                rec.japanese,
            );
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
    use normalizer::{
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
            let _ = writeln!(
                js,
                "\t{{english: `{}`, pekzep_latin: `{}`, pekzep_hanzi: `{}`, chinese_pinyin: `{}`, chinese_hanzi: `{}`, decomposed: `{}`, filetype: `{}`, recording_author: `{}`, pekzep_images: `{}`, japanese: `{}`}},",
                rec.english,
                rec.pekzep_latin,
                remove_guillemets(&rec.pekzep_hanzi),
                capitalize_first_char(&rec.chinese_pinyin),
                normalize_a_b_dialogue(&normalize_chinese_punctuation(&rec.chinese_hanzi)),
                rec.decomposed,
                rec.filetype,
                rec.recording_author,
                convert_hanzi_to_images(&remove_guillemets(&rec.pekzep_hanzi), "() ", "."),
                rec.japanese
            );
        }
    }

    js += "]\n";

    std::fs::write("docs/raw.js", js)?;
    Ok(())
}

/// Generates `char_count.js`
/// # Errors
/// Will return `Err` if the file I/O fails or the render panics.
pub fn write_char_count_js<S: ::std::hash::BuildHasher>(
    char_count: &HashMap<Linzklar, usize, S>,
) -> Result<(), Box<dyn Error>> {
    let mut js = String::from("const CHAR_COUNT = {\n");

    let mut char_count: Vec<_> = char_count.iter().collect();
    char_count.sort_by(|a, b| (b.1, b.0).cmp(&(a.1, a.0)));

    for (k, v) in char_count {
        let _ = writeln!(js, "    \"{k}\": {v},");
    }

    js += "}\n";

    std::fs::write("docs/char_count.js", js)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_split_at_slashslash() {
        use crate::split_at_slashslash;
        assert_eq!(
            split_at_slashslash("行 // 道"),
            (String::from("行 "), String::from(" 道"))
        );
    }
}
