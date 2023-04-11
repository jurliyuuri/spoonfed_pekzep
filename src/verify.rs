use crate::read;
use crate::read::char_pronunciation::{Linzklar, LinzklarString};
use crate::read::vocab::{InternalKey, SplittableCompoundInfo};
use anyhow::anyhow;
use linked_hash_map::LinkedHashMap;
use pekzep_syllable::PekZepSyllable;
use std::collections::HashMap;

pub struct Rows3Item {
    pub syllables: Vec<read::phrase::ExtSyllable>,
    pub decomposition: Vec<Vec<DecompositionItem>>,
    pub row: read::phrase::Item,
}

#[readonly::make]
pub struct DataBundle {
    pub rows3: Vec<Rows3Item>,
    pub vocab_ordered: LinkedHashMap<InternalKey, read::vocab::Item>,
    pub vocab_count: HashMap<InternalKey, usize>,
    pub char_count: HashMap<Linzklar, usize>,
}

impl DataBundle {
    fn check_sentence_pronunciation(
        spoonfed_rows: &LinkedHashMap<Vec<read::phrase::ExtSyllable>, read::phrase::Item>,
        char_pronunciation: &[(Linzklar, pekzep_syllable::PekZepSyllable)],
        contraction_pronunciation: &[(LinzklarString, pekzep_syllable::PekZepSyllable)],
    ) -> anyhow::Result<()> {
        use log::info;
        eprintln!("Checking if the pronunciations of the sentences are correct. Run with RUST_LOG environment variable set to `info` to see the details.");
        for (k, v) in spoonfed_rows.iter() {
            let mut iter = v.pekzep_hanzi.chars();
            let mut key_iter = k.iter();
            while let Some(c) = iter.next() {
                if c.is_whitespace() || c.is_ascii_punctuation() || "！？「」。".contains(c) {
                    info!("Skipped: {}", c);
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
                        return Err(anyhow!(
                            "While trying to match {:?} with {}, end of key encountered",
                            k,
                            v.pekzep_hanzi
                        ));
                    };

                    if let Some(a) = contraction_pronunciation.iter().find(|(h, syllable)| {
                        *h.to_string() == contraction
                            && read::phrase::ExtSyllable::Syllable(*syllable) == expected_syllable
                    }) {
                        info!("matched {} with {}", a.0, a.1);
                    } else {
                        return Err(anyhow!(
                            "While trying to match {:?} with {}, cannot find the contracted pronunciation `{}` for the character sequence `{}`", k, v.pekzep_hanzi,
                            expected_syllable, contraction
                        ));
                    }
                } else if c == 'x' {
                    if Some('i') == iter.next()
                        && Some('z') == iter.next()
                        && Some('i') == iter.next()
                    {
                        if key_iter.next() == Some(&read::phrase::ExtSyllable::Xizi) {
                            info!("matched `xizi`.");
                        } else {
                            return Err(anyhow!("While trying to match {:?} with {}, mismatch found: pekzep_hanzi gave `xizi` but the key was something else", k, v.pekzep_hanzi));
                        }
                    } else {
                        return Err(anyhow!("While trying to match {:?} with {}, expected `xizi` because `x` was encountered, but did not find it.", k, v.pekzep_hanzi));
                    }
                } else {
                    let expected_syllable = if let Some(s) = key_iter.next() {
                        *s
                    } else {
                        return Err(anyhow!(
                            "While trying to match {:?} with {}, end of key encountered",
                            k,
                            v.pekzep_hanzi
                        ));
                    };
                    if let Some(a) = char_pronunciation.iter().find(|(h, syllable)| {
                        *h.to_string() == c.to_string()
                            && read::phrase::ExtSyllable::Syllable(*syllable) == expected_syllable
                    }) {
                        info!("matched {} with {}", a.0, a.1);
                    } else {
                        return Err(anyhow!(
                            "While trying to match {:?} with {}, cannot find the pronunciation `{}` for character `{}`", k, v.pekzep_hanzi,
                            expected_syllable, c
                        ));
                    }
                }
            }

            if let Some(a) = key_iter.next() {
                return Err(anyhow!(
                    "Encountered {} but `{}` ended earlier.\n\nThis occurred while trying to match {:?} with {}, ",
                    a, v.pekzep_hanzi, k, v.pekzep_hanzi,
                ));
            }
        }
        Ok(())
    }

    fn char_count(
        spoonfed_rows: &LinkedHashMap<Vec<read::phrase::ExtSyllable>, read::phrase::Item>,
    ) -> anyhow::Result<HashMap<Linzklar, usize>> {
        use log::info;
        let mut ans = HashMap::new();
        for (_, v) in spoonfed_rows.iter() {
            let mut iter = v.pekzep_hanzi.chars();
            while let Some(c) = iter.next() {
                if c.is_whitespace() || c.is_ascii_punctuation() || "！？「」。".contains(c) {
                    info!("Skipped: {}", c);
                } else if c == '«' {
                    // Handle exceptional contractions such as «足手» xiop1
                    let mut c = iter.next().expect("Unmatched guillemet");
                    loop {
                        if c == '»' {
                            break;
                        }
                        let key = Linzklar::from_char(c)?;
                        let count = ans.entry(key).or_insert(0_usize);
                        *count += 1;
                        c = iter.next().expect("Unmatched guillemet");
                    }
                } else if c == 'x' {
                    if Some('i') == iter.next()
                        && Some('z') == iter.next()
                        && Some('i') == iter.next()
                    {
                    } else {
                        return Err(anyhow!("xizi expected, but found something else."));
                    }
                } else {
                    let key = Linzklar::from_char(c)?;
                    let count = ans.entry(key).or_insert(0_usize);
                    *count += 1;
                }
            }
        }
        Ok(ans)
    }

    fn match_xizi(hanzi_iter: &mut std::str::Chars, v: &read::vocab::Item) -> anyhow::Result<()> {
        use log::info;
        match hanzi_iter.next() {
            Some('x') => {
                if hanzi_iter.next() == Some('i')
                    && hanzi_iter.next() == Some('z')
                    && hanzi_iter.next() == Some('i')
                {
                    info!("matched `xizi` with `xizi`");
                }
            }
            Some(' ') => {
                if hanzi_iter.next() == Some('x')
                    && hanzi_iter.next() == Some('i')
                    && hanzi_iter.next() == Some('z')
                    && hanzi_iter.next() == Some('i')
                {
                    info!("matched `xizi` with `xizi`");
                }
            }
            _ => {
                return Err(anyhow!(
                    "While trying to match {:?} with {}, cannot find matching xizi.",
                    v.pekzep_hanzi,
                    v.pekzep_latin
                ))
            }
        }
        Ok(())
    }

    fn check_vocab_pronunciation(
        vocab: &HashMap<InternalKey, read::vocab::Item>,
        char_pronunciation: &[(Linzklar, pekzep_syllable::PekZepSyllable)],
        contraction_pronunciation: &[(LinzklarString, pekzep_syllable::PekZepSyllable)],
    ) -> anyhow::Result<()> {
        use log::info;
        eprintln!("Checking if the pronunciations of the glosses are correct. Run with RUST_LOG environment variable set to `info` to see the details.");
        // let mut pronunciation_errors_in_vocab = vec![];
        for (_, v) in vocab.iter() {
            if v.pekzep_hanzi == "∅" && v.pekzep_latin.is_empty() {
                info!("matched `∅` with an empty string");
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
                    if c == '«' {
                        // Handle exceptional contractions such as «足手» xiop1
                        let mut contraction = String::new();
                        {
                            let mut c = hanzi_iter.next().expect("Unmatched guillemet");
                            loop {
                                if c == '»' {
                                    break;
                                }
                                contraction.push(c);
                                c = hanzi_iter.next().expect("Unmatched guillemet");
                            }
                        }
                        if let Some(a) = contraction_pronunciation
                            .iter()
                            .find(|(h, sy)| *h.to_string() == contraction && *sy == syllable)
                        {
                            info!("matched {} with {}", a.0, a.1);
                        } else {
                            return Err(anyhow!(
                            "While trying to match {} with {}, cannot find the contracted pronunciation `{}` for `«{}»`", syllable, contraction,
                            syllable, c
                        ));
                        }
                    } else if let Some(a) = char_pronunciation
                        .iter()
                        .find(|(h, sy)| *h.to_string() == c.to_string() && *sy == syllable)
                    {
                        info!("matched {} with {}", a.0, a.1);
                    } else {
                        return Err(anyhow!(
                            "While trying to match {} with {}, cannot find the pronunciation `{}` for character `{}`", v.pekzep_hanzi, v.pekzep_latin,
                            syllable, c
                        ));
                    }
                } else if s == "//" {
                    if hanzi_iter.next() == Some(' ')
                        && hanzi_iter.next() == Some('/')
                        && hanzi_iter.next() == Some('/')
                        && hanzi_iter.next() == Some(' ')
                    {
                        info!("matched `//` with `//`");
                    }
                } else if s == "S" {
                    if hanzi_iter.next() == Some('S') && hanzi_iter.next() == Some(' ') {
                        info!("matched `S` with `S`");
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
                                    Some(_) => panic!("Trying to match {} with {}: Unexpected char {s:?} found while dealing with braces", 
                                    v.pekzep_hanzi, v.pekzep_latin)
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
    fn check_nonrecommended_character(s: &str, variants: &HashMap<Linzklar, Linzklar>) {
        use log::warn;
        for (key, value) in variants {
            if s.contains(&key.to_string()) {
                warn!(
                    "{} contains {}, which should be replaced with {}",
                    s, key, value
                );
            }
        }
    }

    fn check_kan1(pekzep_hanzi: &str, english: &str) {
        use log::warn;
        if pekzep_hanzi.contains('躍') {
            if english.contains("jump") || english.contains("dance") {
                return;
            }
            warn!(
                "{} contains 躍, but the English translation did not contain the word 'jump' or 'dance'. Please check if the sentence `{}` should contains the notion of 'jump'.", 
                pekzep_hanzi,
                english
            );
        }
    }
    fn check_co1(pekzep_hanzi: &str, english: &str) {
        use log::warn;
        if pekzep_hanzi.contains('壁') {
            if english.contains("wall") {
                return;
            }
            warn!(
                "{} contains 壁, but the English translation did not contain the word 'wall'. Please check if the sentence `{}` should contains the notion of 'jump'.", 
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
            );
        }

        // 【噫】の後に句読点も)もないなら警告
        lazy_static! {
            static ref RE: Regex = Regex::new("噫[^)。！？」]").unwrap();
        }
        if RE.is_match(s) {
            warn!(
                "no punctuation found after `噫` in `{}`. Maybe replace it with `之`.",
                s
            );
        }
    }

    fn check_space_before_punctuation(s: &str) {
        use log::warn;
        if s.contains(" .") {
            warn!(
                "a space before a period is detected in `{}`. Remove the space.",
                s
            );
        }

        if s.contains(" ,") {
            warn!(
                "a space before a comma is detected in `{}`. Remove the space.",
                s
            );
        }
    }

    /// # Errors
    /// Returns `Err` if the validation fails.
    pub fn new() -> anyhow::Result<Self> {
        use log::{info, warn};
        use match_pinyin_with_hanzi::match_pinyin_with_hanzi;
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
            Self::check_kan1(&item.pekzep_hanzi, &item.english);
            Self::check_co1(&item.pekzep_hanzi, &item.english);

            Self::check_space_before_punctuation(&item.pekzep_latin);
            Self::check_space_before_punctuation(&item.english);
            Self::check_space_before_punctuation(&item.chinese_pinyin);

            info!("parsing pinyin {:?}:", &item.chinese_pinyin);
            if let Err(err) = match_pinyin_with_hanzi(&item.chinese_pinyin, &item.chinese_hanzi) {
                warn!("{}", err);
            }
        }

        let vocab = read::vocab::parse()?;
        Self::check_vocab_pronunciation(&vocab, &char_pronunciation, &contraction_pronunciation)?;

        for item in vocab.values() {
            Self::check_nonrecommended_character(&item.pekzep_hanzi, &variants);
        }

        let mut vocab_ordered = LinkedHashMap::new();
        let mut vocab_count = HashMap::new();

        let rows3 = spoonfed_rows
            .iter()
            .map(|(syllables, row)| match verify_decomposed(&vocab, row) {
                Ok(decomposition) => {
                    for DecompositionItem {
                        key,
                        voc,
                        splittable_compound_info: _,
                    } in decomposition.iter().flatten()
                    {
                        if !vocab_ordered.contains_key(key) {
                            vocab_ordered.insert((*key).clone(), voc.clone());
                        }

                        let count = vocab_count.entry((*key).clone()).or_insert(0_usize);
                        *count += 1;
                    }
                    Ok(Rows3Item {
                        syllables: syllables.clone(),
                        decomposition,
                        row: row.clone(),
                    })
                }
                Err(e) => Err(e),
            })
            .collect::<anyhow::Result<_>>()?;

        for key in vocab.keys() {
            if !vocab_ordered.contains_key(key) {
                warn!("Item with internal key `{}` is never used", key);
            }
        }

        let char_count = Self::char_count(&spoonfed_rows)?;

        Ok(Self {
            rows3,
            vocab_ordered,
            vocab_count,
            char_count,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DecompositionItem {
    pub key: InternalKey,
    pub voc: read::vocab::Item,
    pub splittable_compound_info: Option<SplittableCompoundInfo>,
}

/// Checks if:
/// * all the morphemes listed in `row.decomposed` are in the vocab list
/// * the `row.decomposed` really is a decomposition of `row.pekzep_hanzi`.
fn verify_decomposed(
    vocab: &HashMap<InternalKey, read::vocab::Item>,
    row: &read::phrase::Item,
) -> anyhow::Result<Vec<Vec<DecompositionItem>>> {
    if row.decomposed.is_none() {
        Ok(vec![])
    } else {
        let rejoined = row.decomposed.as_ref().unwrap().to_plaintext();
        let expectation = row
            .pekzep_hanzi
            .to_string()
            .replace(['！', '？', '。', '「', '」'], "");
        if rejoined != expectation {
            return Err(anyhow!(
                "mismatch: the original row gives {} but the decomposition is {}",
                expectation,
                rejoined
            ));
        }
        let debug_string = row.decomposed.as_ref().unwrap().to_debugtext();
        Ok(vec![row
            .decomposed
            .as_ref()
            .unwrap()
            .0
            .iter()
            .map(|key_gloss| {
                let key = key_gloss.to_internal_key();
                let splittable_compound_info = key_gloss.to_splittable_compound_info();
                let res = vocab.get(&key).ok_or(anyhow! {
                    format!(
                        "Cannot find key {} in the vocab list, found while analyzing {}",
                        &key,
                        debug_string
                    )
                });

                Ok(DecompositionItem {
                    key,
                    voc: res?.clone(),
                    splittable_compound_info,
                })
            })
            .collect::<anyhow::Result<_>>()?])
    }
}
