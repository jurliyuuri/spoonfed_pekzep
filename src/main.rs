#![warn(clippy::pedantic)]
#![allow(clippy::non_ascii_literal)]

use env_logger;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
mod read;
mod verify;

impl read::vocab::Item {
    pub fn to_tab_separated(&self, rel_path: &'static str) -> String {
        self.to_tab_separated_with_custom_linzifier(|s| {
            convert_hanzi_to_images(s, "/{} N()SL", rel_path)
        })
    }
}

use askama::Template;

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
    pekzep_imgs: &'a str,
    author_color: &'a str,
    author_name: &'a str,
    has_audio: bool,
}

#[derive(Template)]
#[template(path = "ind.html")]
struct IndTemplate<'a> {
    index: &'a str,
}

#[derive(Template)]
#[template(path = "vocab.html")]
struct VocabTemplate<'a> {
    analysis: &'a str,
}

#[derive(Template)]
#[template(path = "vocab_list.html")]
struct VocabListTemplate<'a> {
    vocab_html: &'a str,
}

mod filters {
    pub fn capitalizefirstchar(s: &str) -> ::askama::Result<String> {
        let mut v: Vec<char> = s.chars().collect();
        v[0] = v[0].to_uppercase().next().unwrap();
        let s2: String = v.into_iter().collect();
        Ok(s2)
    }
    pub fn linebreaksandtabs(s: &str) -> ::askama::Result<String> {
        let s = s.to_string();
        Ok(format!(
            "<table border=1 cellpadding=5 cellspacing=0>\n\t<tr><td>{}</td></tr>\n</table>",
            s.replace("\t", "</td><td>")
                .replace("\n", "</td></tr>\n\t<tr><td>")
        ))
    }
}

fn to_check(a: bool) -> &'static str {
    if a {
        "&#x2713;"
    } else {
        ""
    }
}

fn convert_hanzi_to_images(s: &str, exclude_list: &str, rel_path: &'static str) -> String {
    let mut ans = String::new();
    let mut iter = s.chars();
    while let Some(c) = iter.next() {
        if c == '∅' {
            ans.push_str(&format!(
                r#"<img src="{}/char_img/blank.png" height="30">"#,
                rel_path
            ))
        } else if c == 'x' {
            if Some('i') == iter.next() && Some('z') == iter.next() && Some('i') == iter.next() {
                ans.push_str(&format!(r#"<img src="{}/char_img/xi.png" height="30"><img src="{}/char_img/zi.png" height="30">"#, rel_path, rel_path))
            } else {
                panic!("Expected `xizi` because `x` was encountered, but did not find it.")
            }
        } else if exclude_list.contains(c) {
            ans.push(c);
        } else {
            ans.push_str(&format!(
                r#"<img src="{}/char_img/{}.png" height="30">"#,
                rel_path, c
            ))
        }
    }

    ans
}

fn generate_phrases(data_bundle: &verify::DataBundle) -> Result<(), Box<dyn Error>> {
    use log::warn;
    eprintln!("Generating phrase/");
    for (i, verify::Rows3Item { sylls, decomp, row }) in data_bundle.rows3.iter().enumerate() {
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
            read::phrase::sylls_to_str_underscore(&sylls)
        ))?;
        let analysis = decomp
            .iter()
            .map(|(_, voc)| voc.to_tab_separated(".."))
            .collect::<Vec<_>>();
        let content = PhraseTemplate {
            english: &row.english,
            chinese_pinyin: &row.chinese_pinyin,
            chinese_hanzi: &row.chinese_hanzi,
            pekzep_latin: &row.pekzep_latin,
            pekzep_hanzi: &row.pekzep_hanzi,
            prev_link: &match prev {
                None => "../index".to_string(),
                Some(verify::Rows3Item { sylls, .. }) => {
                    read::phrase::sylls_to_str_underscore(&sylls)
                }
            },
            next_link: &match next {
                None => "../index".to_string(),
                Some(verify::Rows3Item { sylls, .. }) => {
                    read::phrase::sylls_to_str_underscore(&sylls)
                }
            },
            wav_tag: &if row.filetype.contains(&read::phrase::FilePathType::Wav)
                || row.filetype.contains(&read::phrase::FilePathType::WavR)
            {
                let filename = if row.filetype.contains(&read::phrase::FilePathType::WavR) {
                    read::phrase::sylls_to_rerrliratixka_no_space(&sylls)
                } else {
                    read::phrase::sylls_to_str_underscore(&sylls)
                };

                if !std::path::Path::new(&format!("docs/spoonfed_pekzep_sounds/{}.wav", filename))
                    .exists()
                {
                    warn!("wav file not found: {}.wav", filename)
                }
                format!(
                    r#"<source src="../spoonfed_pekzep_sounds/{}.wav" type="audio/wav">"#,
                    filename
                )
            } else {
                "".to_owned()
            },
            oga_tag: &if row.filetype.contains(&read::phrase::FilePathType::Oga) {
                let filename = read::phrase::sylls_to_str_underscore(&sylls);
                if !std::path::Path::new(&format!("docs/spoonfed_pekzep_sounds/{}.oga", filename))
                    .exists()
                {
                    warn!("oga file not found: {}.oga", filename)
                }
                format!(
                    r#"<source src="../spoonfed_pekzep_sounds/{}.oga" type="audio/ogg">"#,
                    filename
                )
            } else {
                "".to_owned()
            },
            analysis: &analysis.join("\n"),
            pekzep_imgs: &convert_hanzi_to_images(&row.pekzep_hanzi, "() ", ".."),
            author_color: &if row.recording_author == Some(read::phrase::Author::JektoVatimeliju) {
                "#754eab"
            } else if row.recording_author == Some(read::phrase::Author::FaliraLyjotafis) {
                "#e33102"
            } else {
                if row.recording_author.is_some() {
                    warn!("Unrecognized author `{:?}`", row.recording_author);
                }
                "#000000"
            },
            author_name: &match &row.recording_author {
                Some(author) => format!("{}", author),
                None => "".to_string(),
            },
            has_audio: row.recording_author.is_some(),
        };
        write!(file, "{}", content.render().unwrap())?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    use std::env;
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "warn");
    }
    env_logger::init();
    let data_bundle = verify::DataBundle::new()?;

    generate_phrases(&data_bundle)?;

    eprintln!("Generating vocab/");
    for (key, v) in &data_bundle.vocab_ordered {
        let mut file = File::create(format!(
            "docs/vocab/{}.html",
            key.replace(" // ", "_slashslash_")
        ))?;
        write!(
            file,
            "{}",
            VocabTemplate {
                analysis: &v.to_tab_separated("..")
            }
            .render()
            .unwrap()
        )?;
    }

    eprintln!("Generating vocab_list_internal.html");
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
        .render()
        .unwrap()
    )?;

    eprintln!("Generating vocab_list.html");
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
        .render()
        .unwrap()
    )?;

    eprintln!("Generating index.html");
    let mut file = File::create("docs/index.html")?;
    let mut index = vec!["<abbr title=\"Audio available in Edge, Firefox, Chrome and Opera. / 在Edge、Firefox、Chrome和Opera中都可以听到录音。\">🔊<i class=\"fab fa-chrome\"></i><i class=\"fab fa-firefox-browser\"></i><i class=\"fab fa-edge\"></i><i class=\"fab fa-edge-legacy\"></i><i class=\"fab fa-opera\"></i></abbr>\t<abbr title=\"Audio available in Safari. / 在Safari中都可以听到录音。\">🔊<i class=\"fab fa-safari\"></i></abbr>\tgloss\tphrase".to_string()];
    for verify::Rows3Item { sylls, decomp, row } in &data_bundle.rows3 {
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
            to_check(!decomp.is_empty()),
            read::phrase::sylls_to_str_underscore(&sylls),
            row.pekzep_latin
        ));
    }

    write!(
        file,
        "{}",
        IndTemplate {
            index: &index.join("\n")
        }
        .render()
        .unwrap()
    )?;

    Ok(())
}
