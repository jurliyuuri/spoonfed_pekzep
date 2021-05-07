use crate::read;
use linked_hash_map::LinkedHashMap;
use partition_eithers::collect_any_errors;
use pekzep_syllable::PekZepSyllable;
use std::collections::HashMap;
use std::error::Error;

pub struct Rows3Item {
    pub syllables: Vec<read::phrase::ExtSyllable>,
    pub decomposition: Vec<(String, read::vocab::Item)>,
    pub row: read::phrase::Item,
}

#[readonly::make]
pub struct DataBundle {
    pub rows3: Vec<Rows3Item>,
    pub vocab_ordered: LinkedHashMap<String, read::vocab::Item>,
}

impl DataBundle {
    fn check_sentence_pronunciation(
        spoonfed_rows: &LinkedHashMap<Vec<read::phrase::ExtSyllable>, read::phrase::Item>,
        char_pronunciation: &[(String, pekzep_syllable::PekZepSyllable)],
        contraction_pronunciation: &[(String, pekzep_syllable::PekZepSyllable)],
    ) -> Result<(), String> {
        use log::info;
        eprintln!("Checking if the pronunciations of the sentences are correct. Run with RUST_LOG environment variable set to `info` to see the details.");
        for (k, v) in spoonfed_rows.iter() {
            let mut iter = v.pekzep_hanzi.chars();
            let mut key_iter = k.iter();
            while let Some(c) = iter.next() {
                if c.is_whitespace() || c.is_ascii_punctuation() || "！？「」。".contains(c) {
                    info!("Skipped: {}", c)
                } else if c == '«' {
                    // Handle exceptional contractions such as «足手» xiop1
                    let mut contraction = String::new();
                    {
                        let mut c = iter.next().expect("Unmatched guillemet");
                        loop {
                            if c == '»' {
                                break;
                            }
                            contraction.push(c);
                            c = iter.next().expect("Unmatched guillemet");
                        }
                    }

                    let expected_syllable = if let Some(s) = key_iter.next() {
                        *s
                    } else {
                        return Err(format!(
                            "While trying to match {:?} with {}, end of key encountered",
                            k, v.pekzep_hanzi
                        ));
                    };

                    if let Some(a) = contraction_pronunciation.iter().find(|(h, syllable)| {
                        **h == contraction
                            && read::phrase::ExtSyllable::Syllable(*syllable) == expected_syllable
                    }) {
                        info!("matched {} with {}", a.0, a.1)
                    } else {
                        return Err(format!(
                            "While trying to match {:?} with {}, cannot find the contracted pronunciation `{}` for the character sequence `{}`", k, v.pekzep_hanzi,
                            expected_syllable, contraction
                        ));
                    }
                } else if c == 'x' {
                    if Some('i') == iter.next()
                        && Some('z') == iter.next()
                        && Some('i') == iter.next()
                    {
                        if let Some(read::phrase::ExtSyllable::Xizi) = key_iter.next() {
                            info!("matched `xizi`.")
                        } else {
                            return Err(format!("While trying to match {:?} with {}, mismatch found: pekzep_hanzi gave `xizi` but the key was something else", k, v.pekzep_hanzi));
                        }
                    } else {
                        return Err(format!("While trying to match {:?} with {}, expected `xizi` because `x` was encountered, but did not find it.", k, v.pekzep_hanzi));
                    }
                } else {
                    let expected_syllable = if let Some(s) = key_iter.next() {
                        *s
                    } else {
                        return Err(format!(
                            "While trying to match {:?} with {}, end of key encountered",
                            k, v.pekzep_hanzi
                        ));
                    };
                    if let Some(a) = char_pronunciation.iter().find(|(h, syllable)| {
                        **h == c.to_string()
                            && read::phrase::ExtSyllable::Syllable(*syllable) == expected_syllable
                    }) {
                        info!("matched {} with {}", a.0, a.1)
                    } else {
                        return Err(format!(
                            "While trying to match {:?} with {}, cannot find the pronunciation `{}` for character `{}`", k, v.pekzep_hanzi,
                            expected_syllable, c
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn match_xizi(hanzi_iter: &mut std::str::Chars, v: &read::vocab::Item) -> Result<(), String> {
        use log::info;
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
            _ => {
                return Err(format!(
                    "While trying to match {:?} with {}, cannot find matching xizi.",
                    v.pekzep_hanzi, v.pekzep_latin
                ))
            }
        }
        Ok(())
    }

    fn check_vocab_pronunciation(
        vocab: &HashMap<String, read::vocab::Item>,
        char_pronunciation: &[(String, pekzep_syllable::PekZepSyllable)],
    ) -> Result<(), String> {
        use log::info;
        eprintln!("Checking if the pronunciations of the glosses are correct. Run with RUST_LOG environment variable set to `info` to see the details.");
        // let mut pronunciation_errors_in_vocab = vec![];
        for (_, v) in vocab.iter() {
            if v.pekzep_hanzi == "∅" && v.pekzep_latin.is_empty() {
                info!("matched `∅` with an empty string")
            }
            let mut latin_iter = v.pekzep_latin.split(char::is_whitespace);
            let mut hanzi_iter = v.pekzep_hanzi.chars();
            'a: while let Some(s) = latin_iter.next() {
                if s == "xizi" {
                    Self::match_xizi(&mut hanzi_iter, v)?;
                }

                if let Some(syllable) = PekZepSyllable::parse(s) {
                    let mut c = hanzi_iter.next().expect("Unmatched syllable");
                    loop {
                        if !c.is_whitespace() {
                            break;
                        }
                        c = hanzi_iter.next().expect("Unmatched syllable");
                    }
                    if let Some(a) = char_pronunciation
                        .iter()
                        .find(|(h, sy)| **h == c.to_string() && *sy == syllable)
                    {
                        info!("matched {} with {}", a.0, a.1)
                    } else {
                        return Err(format!(
                            "While trying to match {:?} with {}, cannot find the pronunciation `{}` for character `{}`", v.pekzep_hanzi, v.pekzep_latin,
                            syllable, c
                        ));
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
                                    Some(_) => panic!("Trying to match {} with {}: Unexpected char {:?} found while dealing with braces", 
                                    v.pekzep_hanzi, v.pekzep_latin, s)
                                }
                            }
                            loop {
                                match hanzi_iter.next() {
                                    Some('}') => break,
                                    None => panic!("Unexpected end of the input"),
                                    Some(_) => continue,
                                }
                            }
                        }
                        Some(_) => continue,
                        None => break,
                    }
                }
            }
        }
        Ok(())
    }
    fn check_nonrecommended_character(s: &str, variants: &HashMap<String, String>) {
        use log::warn;
        for (key, value) in variants {
            if s.contains(key) {
                warn!(
                    "{} contains {}, which should be replaced with {}",
                    s, key, value
                );
            }
        }
    }

    fn check_kan(pekzep_hanzi: &str, english: &str) {
        use log::warn;
        if pekzep_hanzi.contains('躍') {
            if english.contains("jump") {
                return;
            }
            warn!(
                "{} contains 躍, but the English translation did not contain the word 'jump'. Please check if the sentence `{}` should contains the notion of 'jump'.", 
                pekzep_hanzi,
                english
            );
        }
    }
    fn check_a(s: &str) {
        use log::warn;
        use regex::Regex;

        // 【之】の後に句読点あるなら警告
        if s.contains("之。") || s.contains("之！") || s.contains("之？") || s.contains("之」")
        {
            warn!(
                "punctuation after `之` is detected in `{}`. Maybe replace it with `噫`?",
                s
            )
        }

        // 【噫】の後に句読点も)もないなら警告
        lazy_static! {
            static ref RE: Regex = Regex::new("噫[^)。！？」]").unwrap();
        }
        if RE.is_match(s) {
            warn!(
                "no punctuation found after `噫` in `{}`. Maybe replace it with `之`.",
                s
            )
        }
    }
    pub fn new() -> Result<Self, Box<dyn Error>> {
        use log::warn;
        let (char_pronunciation, variants) = read::char_pronunciation::parse()?;
        let contraction_pronunciation = read::contraction::parse()?;

        let spoonfed_rows = read::phrase::parse()?;
        Self::check_sentence_pronunciation(
            &spoonfed_rows,
            &char_pronunciation,
            &contraction_pronunciation,
        )?;

        for (_, item) in &spoonfed_rows {
            Self::check_nonrecommended_character(&item.pekzep_hanzi, &variants);
            Self::check_a(&item.pekzep_hanzi);
            Self::check_kan(&item.pekzep_hanzi, &item.english);
        }

        let vocab = read::vocab::parse()?;
        Self::check_vocab_pronunciation(&vocab, &char_pronunciation)?;

        for item in vocab.values() {
            Self::check_nonrecommended_character(&item.pekzep_hanzi, &variants);
        }

        let mut vocab_ordered = LinkedHashMap::new();

        let rows3 = collect_any_errors(
            spoonfed_rows
                .iter()
                .map(|(syllables, row)| {
                    match parse_decomposed(&vocab, row).map_err(|e| e.join("\n")) {
                        Ok(decomposition) => {
                            for (key, voc) in &decomposition {
                                if !vocab_ordered.contains_key(key) {
                                    vocab_ordered.insert(key.to_string(), voc.clone());
                                }
                            }
                            Ok(Rows3Item {
                                syllables: syllables.clone(),
                                decomposition,
                                row: row.clone(),
                            })
                        }
                        Err(e) => Err(e),
                    }
                })
                .collect::<Vec<_>>(),
        )
        .map_err(|e| -> Box<dyn Error> { e.join("\n").into() })?;

        for key in vocab.keys() {
            if !vocab_ordered.contains_key(key) {
                warn!("Item with internal key `{}` is never used", key);
            }
        }

        Ok(Self {
            rows3,
            vocab_ordered,
        })
    }
}

/// Checks if:
/// * all the morphemes listed in `row.decomposed` are in the vocab list
/// * the `row.decomposed` really is a decomposition of `row.pekzep_hanzi`.
fn parse_decomposed(
    vocab: &HashMap<String, read::vocab::Item>,
    row: &read::phrase::Item,
) -> Result<Vec<(String, read::vocab::Item)>, Vec<String>> {
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
