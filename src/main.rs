use csv::StringRecord;
use pekzep_syllable::PekZepSyllable;
use serde_derive::{Deserialize as De, Serialize as Ser};
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

mod read;

#[derive(Ser, De, Debug, Clone)]
struct MainRow {
    english: String,
    pekzep_latin: String,
    pekzep_hanzi: String,
    chinese_pinyin: String,
    chinese_hanzi: String,
    decomposed: String,
    filetype: String,
    recording_author: String,
}

use std::collections::HashMap;

use linked_hash_map::LinkedHashMap;

impl read::vocab::Vocab {
    pub fn to_tab_separated(&self, rel_path: &'static str) -> String {
        self.to_tab_separated_with_custom_linzifier(|s| convert_hanzi_to_images(s, "/{} N()SL", rel_path))
    }
}

fn parse_spoonfed() -> Result<LinkedHashMap<Vec<ExtSyll>, MainRow>, Box<dyn Error>> {
    let f = File::open("raw/Spoonfed Pekzep - SpoonfedPekzep.tsv")?;
    let f = BufReader::new(f);
    let mut rows = LinkedHashMap::new();
    let mut errors = vec![];
    for line in f.lines() {
        // to prevent double quotes from vanishing, I do not read with CSV parser
        let row: MainRow =
            StringRecord::from(line.unwrap().split('\t').collect::<Vec<_>>()).deserialize(None)?;

        let sylls = encode_to_pekzep_syllables(&row.pekzep_latin)?;
        if !sylls.is_empty() && rows.insert(sylls.clone(), row.clone()).is_some() {
            // in HashSet::insert, if the set did have this value present, false is returned.
            errors.push(format!(
                "duplicate phrase detected: {}",
                sylls_to_str_underscore(&sylls)
            ));
        }
    }

    if errors.is_empty() {
        Ok(rows)
    } else {
        let err: Box<dyn Error> = errors.join("\n").into();
        Err(err)
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
    audio_path_oga: &'a str,
    pekzep_imgs: &'a str,
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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum ExtSyll {
    Syll(PekZepSyllable),
    Xizi,
}

impl std::fmt::Display for ExtSyll {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtSyll::Syll(s) => write!(f, "{}", s),
            ExtSyll::Xizi => write!(f, "xizi"),
        }
    }
}

impl ExtSyll {
    fn to_rerrliratixka(&self) -> String {
        match &self {
            ExtSyll::Syll(s) => s.clone().to_rerrliratixka(),
            ExtSyll::Xizi => "xizi".to_string(),
        }
    }
}

fn encode_to_pekzep_syllables(i: &str) -> Result<Vec<ExtSyll>, Box<dyn Error>> {
    error_collector(
        i.split(|c: char| c.is_ascii_punctuation() || c.is_whitespace())
            .filter_map(|k| {
                if k.is_empty() {
                    None
                } else {
                    Some(match PekZepSyllable::parse(k) {
                        Some(s) => Ok(ExtSyll::Syll(s)),
                        None => {
                            if k == "xizi" {
                                Ok(ExtSyll::Xizi)
                            } else {
                                Err(format!("Failed to parse a pekzep syllable {}", k))
                            }
                        }
                    })
                }
            })
            .collect::<Vec<_>>(),
    )
    .map_err(|e| e.join("\n").into())
}

fn sylls_to_rerrliratixka_no_space(sylls: &[ExtSyll]) -> String {
    sylls
        .iter()
        .map(ExtSyll::to_rerrliratixka)
        .collect::<Vec<_>>()
        .join("")
}

fn sylls_to_str_underscore(sylls: &[ExtSyll]) -> String {
    sylls
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>()
        .join("_")
}

fn to_check(a: bool) -> &'static str {
    if a {
        "&#x2713;"
    } else {
        ""
    }
}

// return Ok if all are Ok
fn error_collector<T, E>(a: Vec<Result<T, E>>) -> Result<Vec<T>, Vec<E>> {
    let mut ts = Vec::new();
    let mut es = Vec::new();
    for q in a {
        match q {
            Ok(t) => ts.push(t),
            Err(e) => es.push(e),
        }
    }
    if es.is_empty() {
        Ok(ts)
    } else {
        Err(es)
    }
}

/// Checks if:
/// * all the morphemes listed in `row.decomposed` are in the vocab list
/// * the `row.decomposed` really is a decomposition of `row.pekzep_hanzi`.
fn parse_decomposed(
    vocab: &HashMap<String, read::vocab::Vocab>,
    row: &MainRow,
) -> Result<Vec<(String, read::vocab::Vocab)>, Vec<String>> {
    if row.decomposed.is_empty() {
        Ok(vec![])
    } else {
        let rejoined = row
            .decomposed
            .split('.')
            .map(|a| {
                let init_char = a.chars().next().unwrap();
                if init_char == '∅' {
                    return "".to_string();
                }
                if a.contains('#') {
                    return a.chars().take_while(|c| *c != '#').collect::<String>();
                }
                if a.contains('!') {
                    let mut iter = a.chars().skip_while(|c| *c != '!');
                    iter.next();
                    return iter.collect::<String>();
                }

                // handle xizi
                if init_char.is_ascii_alphabetic() {
                    // drop only numeric characters from the end of the string
                    let rev = a
                        .chars()
                        .rev()
                        .skip_while(|c| c.is_numeric())
                        .collect::<String>();
                    rev.chars().rev().collect::<String>()
                } else {
                    // drop only alphanumeric characters from the end of the string
                    let rev = a
                        .chars()
                        .rev()
                        .skip_while(char::is_ascii_alphanumeric)
                        .collect::<String>();
                    rev.chars().rev().collect::<String>()
                }
            })
            .collect::<String>();
        let expectation = row
            .pekzep_hanzi
            .to_string()
            .replace("！", "")
            .replace("？", "")
            .replace("。", "")
            .replace("「", "")
            .replace("」", "");
        if rejoined != expectation {
            return Err(vec![format!(
                "mismatch: the original row gives {} but the decomposition is {}",
                expectation, rejoined
            )]);
        }
        error_collector(
            row.decomposed
                .split('.')
                .map(|a| {
                    let key = a.to_string().replace("!", " // ").replace("#", " // ");
                    let res = vocab.get(&key).ok_or(format!(
                        "Cannot find key {} in the vocab list, found while analyzing {}",
                        key, row.decomposed
                    ));
                    Ok((key, res?.to_owned()))
                })
                .collect::<Vec<_>>(),
        )
    }
}

fn convert_hanzi_to_images(s: &str, exclude_list: &str, rel_path: &'static str) -> String {
    let mut ans = String::new();
    let mut iter = s.chars();
    while let Some(c) = iter.next() {
        if c == '∅' {
            ans.push_str(&format!(r#"<img src="{}/char_img/blank.png" width="30" height="30">"#, rel_path))
        } else if c == 'x' {
            if Some('i') == iter.next() && Some('z') == iter.next() && Some('i') == iter.next() {
                ans.push_str(&format!(r#"<img src="{}/char_img/xi.png" width="30" height="30"><img src="{}/char_img/zi.png" width="30" height="30">"#, rel_path, rel_path))
            } else {
                panic!("Expected `xizi` because `x` was encountered, but did not find it.")
            }
        } else if exclude_list.contains(c) {
            ans.push(c);
        } else {
            ans.push_str(&format!(
                r#"<img src="{}/char_img/{}.png" width="30" height="30">"#,
                rel_path,
                c
            ))
        }
    }

    ans
}

struct Foo {
    vocab: HashMap<String, read::vocab::Vocab>,
    rows3: Vec<(Vec<ExtSyll>, Vec<(String, read::vocab::Vocab)>, MainRow)>,
    vocab_ordered: LinkedHashMap<String, read::vocab::Vocab>,
}

impl Foo {
    pub fn new() -> Result<Foo, Box<dyn Error>> {
        let spoonfed_rows = parse_spoonfed()?;

        let vocab = read::vocab::parse_vocabs()?;
        let mut vocab_ordered = LinkedHashMap::new();

        let rows3 = error_collector(
            spoonfed_rows
                .iter()
                .map(
                    |(sylls, row)| match parse_decomposed(&vocab, row).map_err(|e| e.join("\n")) {
                        Ok(decomp) => {
                            for (key, voc) in &decomp {
                                if !vocab_ordered.contains_key(key) {
                                    vocab_ordered.insert(key.to_string(), voc.clone());
                                }
                            }
                            Ok((sylls.clone(), decomp, row.clone()))
                        }
                        Err(e) => Err(e),
                    },
                )
                .collect::<Vec<_>>(),
        )
        .map_err(|e| -> Box<dyn Error> { e.join("\n").into() })?;

        Ok(Foo {
            vocab,
            rows3,
            vocab_ordered,
        })
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let foo = Foo::new()?;
    for (i, (sylls, decomp, this)) in foo.rows3.iter().enumerate() {
        let prev = if i == 0 { None } else { foo.rows3.get(i - 1) };
        let next = foo.rows3.get(i + 1);
        if this.pekzep_latin.is_empty() {
            continue;
        }
        let mut file = File::create(format!(
            "docs/phrase/{}.html",
            sylls_to_str_underscore(&sylls)
        ))?;
        let analysis = decomp
            .iter()
            .map(|(_, voc)| voc.to_tab_separated(".."))
            .collect::<Vec<_>>();
        let content = PhraseTemplate {
            english: &this.english,
            chinese_pinyin: &this.chinese_pinyin,
            chinese_hanzi: &this.chinese_hanzi,
            pekzep_latin: &this.pekzep_latin,
            pekzep_hanzi: &this.pekzep_hanzi,
            prev_link: &match prev {
                None => "../index".to_string(),
                Some((sylls, _, _)) => sylls_to_str_underscore(&sylls),
            },
            next_link: &match next {
                None => "../index".to_string(),
                Some((sylls, _, _)) => sylls_to_str_underscore(&sylls),
            },
            wav_tag: &if this.filetype.contains("wav") {
                format!(
                    r#"<source src="../spoonfed_pekzep_sounds/{}.wav" type="audio/wav">"#,
                    sylls_to_rerrliratixka_no_space(&sylls)
                )
            } else {
                "".to_owned()
            },
            analysis: &analysis.join("\n"),
            audio_path_oga: &sylls_to_str_underscore(&sylls),
            pekzep_imgs: &convert_hanzi_to_images(&this.pekzep_hanzi, "() ", ".."),
        };
        write!(file, "{}", content.render().unwrap())?;
    }

    for (key, v) in foo.vocab {
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

    let mut vocab_file = File::create("docs/vocab_list_internal.html")?;
    let mut vocab_html = vec![];
    for (key, vocab) in &foo.vocab_ordered {
        vocab_html.push(format!("{}\t{}", key, vocab.to_tab_separated(".")))
    }
    write!(
        vocab_file, "{}", VocabListTemplate { vocab_html: &vocab_html.join("\n") }.render().unwrap()
    )?;

    let mut vocab_file = File::create("docs/vocab_list.html")?;
    let mut vocab_html = vec![];
    for (_, vocab) in foo.vocab_ordered {
        vocab_html.push(vocab.to_tab_separated("."))
    }
    write!(
        vocab_file, "{}", VocabListTemplate { vocab_html: &vocab_html.join("\n") }.render().unwrap()
    )?;

    let mut file = File::create("docs/index.html")?;
    let mut index = vec!["<abbr title=\"Audio available in Edge, Firefox, Chrome and Opera. / 在Edge、Firefox、Chrome和Opera中都可以听到录音。\">🔊<i class=\"fab fa-chrome\"></i><i class=\"fab fa-firefox-browser\"></i><i class=\"fab fa-edge\"></i><i class=\"fab fa-edge-legacy\"></i><i class=\"fab fa-opera\"></i></abbr>\t<abbr title=\"Audio available in Safari. / 在Safari中都可以听到录音。\">🔊<i class=\"fab fa-safari\"></i></abbr>\tgloss\tphrase".to_string()];
    for (sylls, decomp, r) in foo.rows3 {
        index.push(format!(
            "{}\t{}\t{}\t<a href=\"phrase/{}.html\">{}</a>",
            to_check(r.filetype.contains("wav") || r.filetype.contains("oga")),
            to_check(r.filetype.contains("wav")),
            to_check(!decomp.is_empty()),
            sylls_to_str_underscore(&sylls),
            r.pekzep_latin
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
