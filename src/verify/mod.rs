use crate::read;
use linked_hash_map::LinkedHashMap;
use partition_eithers::collect_any_errors;
use pekzep_syllable::PekZepSyllable;
use std::collections::HashMap;
use std::error::Error;

#[readonly::make]
pub struct DataBundle {
    pub rows3: Vec<(
        Vec<read::main_row::ExtSyll>,
        Vec<(String, read::vocab::Vocab)>,
        read::main_row::MainRow,
    )>,
    pub vocab_ordered: LinkedHashMap<String, read::vocab::Vocab>,
}

impl DataBundle {
    pub fn new() -> Result<DataBundle, Box<dyn Error>> {
        use log::{info, warn};
        let char_pronunciation = read::char_pronunciation::parse()?;

        let spoonfed_rows = read::main_row::parse()?;

        eprintln!("Checking if the pronunciations of the sentences are correct. Run with RUST_LOG environment variable set to `info` to see the details.");
        for (k, v) in spoonfed_rows.iter() {
            let mut iter = v.pekzep_hanzi.chars();
            let mut key_iter = k.iter();
            while let Some(c) = iter.next() {
                if c.is_whitespace() || c.is_ascii_punctuation() || "！？「」。".contains(c) {
                    info!("Skipped: {}", c)
                } else if c == 'x' {
                    if Some('i') == iter.next()
                        && Some('z') == iter.next()
                        && Some('i') == iter.next()
                    {
                        if let Some(read::main_row::ExtSyll::Xizi) = key_iter.next() {
                            info!("matched `xizi`.")
                        } else {
                            panic!("While trying to match {:?} with {}, mismatch found: pekzep_hanzi gave `xizi` but the key was something else", k, v.pekzep_hanzi)
                        }
                    } else {
                        panic!("While trying to match {:?} with {}, expected `xizi` because `x` was encountered, but did not find it.", k, v.pekzep_hanzi)
                    }
                } else {
                    let expected_syll = if let Some(s) = key_iter.next() {
                        *s
                    } else {
                        panic!(
                            "While trying to match {:?} with {}, end of key encountered",
                            k, v.pekzep_hanzi
                        )
                    };
                    if let Some(a) = char_pronunciation.iter().find(|(h, syll)| {
                        *h == c.to_string() && read::main_row::ExtSyll::Syll(*syll) == expected_syll
                    }) {
                        info!("matched {} with {}", a.0, a.1)
                    } else {
                        panic!(
                            "While trying to match {:?} with {}, cannot find the pronunciation `{}` for character `{}`", k, v.pekzep_hanzi,
                            expected_syll, c
                        )
                    }
                }
            }
        }
        //if !pronunciation_errors_in_spoonfed.is_empty() {
        //    return Err(pronunciation_errors_in_spoonfed.join("\n").into());
        //}

        let vocab = read::vocab::parse()?;
        eprintln!("Checking if the pronunciations of the glosses are correct. Run with RUST_LOG environment variable set to `info` to see the details.");
        // let mut pronunciation_errors_in_vocab = vec![];
        for (_, v) in vocab.iter() {
            if v.pekzep_hanzi == "∅" && v.pekzep_latin == "" {
                info!("matched `∅` with an empty string")
            }
            let mut latin_iter = v.pekzep_latin.split(char::is_whitespace);
            let mut hanzi_iter = v.pekzep_hanzi.chars();
            'a: while let Some(s) = latin_iter.next() {
                if s == "xizi" {
                    match hanzi_iter.next() {
                        Some('x') => {
                            if hanzi_iter.next() == Some('i')
                                && hanzi_iter.next() == Some('z')
                                && hanzi_iter.next() == Some('i')
                            {
                                info!("matched `xizi` with `xizi`")
                            }
                        }
                        Some(' ') => {
                            if hanzi_iter.next() == Some('x')
                                && hanzi_iter.next() == Some('i')
                                && hanzi_iter.next() == Some('z')
                                && hanzi_iter.next() == Some('i')
                            {
                                info!("matched `xizi` with `xizi`")
                            }
                        }
                        _ => panic!(
                            "While trying to match {:?} with {}, cannot find matching xizi.",
                            v.pekzep_hanzi, v.pekzep_latin
                        ),
                    }
                }

                if let Some(syll) = PekZepSyllable::parse(s) {
                    let mut c = hanzi_iter.next().expect("Unmatched syllable");
                    loop {
                        if !c.is_whitespace() {
                            break;
                        }
                        c = hanzi_iter.next().expect("Unmatched syllable");
                    }
                    if let Some(a) = char_pronunciation
                        .iter()
                        .find(|(h, sy)| *h == c.to_string() && *sy == syll)
                    {
                        info!("matched {} with {}", a.0, a.1)
                    } else {
                        panic!(
                            "While trying to match {:?} with {}, cannot find the pronunciation `{}` for character `{}`", v.pekzep_hanzi, v.pekzep_latin,
                            syll, c
                        )
                    }
                } else if s == "//" {
                    if hanzi_iter.next() == Some(' ')
                        && hanzi_iter.next() == Some('/')
                        && hanzi_iter.next() == Some('/')
                        && hanzi_iter.next() == Some(' ')
                    {
                        info!("matched `//` with `//`")
                    }
                } else if s == "S" {
                    if hanzi_iter.next() == Some('S') && hanzi_iter.next() == Some(' ') {
                        info!("matched `S` with `S`")
                    }
                } else {
                    match s.chars().next() {
                        Some('{') => {
                            // for the latin side, start ignoring everything else until the matching '}'
                            let mut u = s;
                            loop {
                                if u.ends_with('}') {
                                    break;
                                }
                                u = latin_iter.next().expect("Unmatched }");
                            }

                            // for the hanzi side, skip
                            loop {
                                match hanzi_iter.next() {
                                    Some(' ') => continue,
                                    Some('{') => break,
                                    None => continue 'a,
                                    s => panic!("Trying to match {} with {}: Unexpected char {:?} found while dealing with braces", 
                                    v.pekzep_hanzi, v.pekzep_latin, s)
                                }
                            }
                            loop {
                                match hanzi_iter.next() {
                                    Some('}') => break,
                                    None => panic!("Unexpected end of the input"),
                                    _ => continue,
                                }
                            }
                        }
                        Some(_) => continue,
                        None => break,
                    }
                }
            }
        }

        let mut vocab_ordered = LinkedHashMap::new();

        let rows3 = collect_any_errors(
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

        for key in vocab.keys() {
            if !vocab_ordered.contains_key(key) {
                warn!("Vocab with internal key `{}` is never used", key);
            }
        }

        Ok(DataBundle {
            rows3,
            vocab_ordered,
        })
    }
}

/// Checks if:
/// * all the morphemes listed in `row.decomposed` are in the vocab list
/// * the `row.decomposed` really is a decomposition of `row.pekzep_hanzi`.
fn parse_decomposed(
    vocab: &HashMap<String, read::vocab::Vocab>,
    row: &read::main_row::MainRow,
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
        collect_any_errors(
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
