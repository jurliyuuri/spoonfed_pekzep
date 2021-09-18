use crate::read;
use linked_hash_map::LinkedHashMap;
use partition_eithers::collect_any_errors;
use pekzep_syllable::PekZepSyllable;
use std::collections::HashMap;
use std::error::Error;

pub struct Rows3Item {
    pub syllables: Vec<read::phrase::ExtSyllable>,
    pub decomposition: Vec<DecompositionItem>,
    pub row: read::phrase::Item,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
/// The key used to identify a word in the glosses. It is made up of a "main" followed by an optional "postfix", where "main" denotes the word and "postfix" disambiguates the subdivision of the word.
///
/// The postfix is `[0-9a-zA-Z]*` when the main does not begin with an ASCII character. The postfix is `:[0-9a-zA-Z]*` if the main *does* begin with an ASCII character.
/// 
/// It must adhere to one of the following formats (Note that, as of 2021-09-18, Note that we only allow CJK Unified Ideographs or CJK Unified Ideographs Extension A to be used as a transcription):
/// 
/// | Main | Optional postfix                                                              | Annotation Removed                     | Example |                                                                                                                                                       |
/// |---------|----------------------------------------------------------------------|----------------------------------------|-------------|-------------------------------------------------------------------------------------------------------------------------------------------------------|
/// |  `[\u3400-\u4DBF\u4E00-\u9FFF]+`       | `[0-9a-zA-Z]*`                        | `種茶銭処`, `紙机戦`, `於dur`, `須多2` |  The most basic form available for a key: the postfix disambiguates which meaning is intended for the glossed word.                                            |
/// |  `∅`      | `[0-9a-zA-Z]*`                                                      | `∅`, `∅3`                              | used when the word is realized as an empty string in Pekzep                                                                                           |
/// |  `[\u3400-\u4DBF\u4E00-\u9FFF]+) // ([\u3400-\u4DBF\u4E00-\u9FFF]+`      | `[0-9a-zA-Z]*` | `享 // 銭`, `行 // 星周`               | used for a splittable compound                                                                                                                        |
/// |  `«[\u3400-\u4DBF\u4E00-\u9FFF]+»`     | `[0-9a-zA-Z]*`                                  | `«足手»`                               |  used when a multisyllable merges into a single syllable                                                                                               |
/// |  `[a-z0-9 ]+` (cannot start or end with a space)     | `:[0-9a-zA-Z]*`                                          | `xizi`, `xizi xizi`                    |  Denotes `xizi`, a postfix used after a name, or `xizi xizi`, an interjection. Currently, this program does not allow any non-Linzklar word other than `xizi`. |
/// |  `[a-z0-9 ]+[\u3400-\u4DBF\u4E00-\u9FFF]+`     | `:[0-9a-zA-Z]*`             | `xizi噫`                               |  Denotes `xizi噫`, an interjection. Currently, this program does not allow any non-Linzklar word other than `xizi`.                                    |
/// |  `\([\u3400-\u4DBF\u4E00-\u9FFF]+\)`     | `:[0-9a-zA-Z]*`                                 | `(噫)`                               |  used for the 噫 placed after 之 to mark that the sentence ends with a possessive                                                                                              |
pub struct VocabInternalKey(String);

impl VocabInternalKey {
    /// `享 // 銭` → `享_slashslash_銭`
    #[must_use]
    pub fn to_path_safe_string(&self) -> String {
        self.0.replace(" // ", "_slashslash_")
    }

    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }

    pub fn to_str(&self) -> &str {
        &self.0
    }

    #[must_use]
    fn new(a: &str) -> Self {
        Self(a.to_owned())
    }
}

#[readonly::make]
pub struct DataBundle {
    pub rows3: Vec<Rows3Item>,
    pub vocab_ordered: LinkedHashMap<VocabInternalKey, read::vocab::Item>,
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
                        return Err(format!(
                            "While trying to match {:?} with {}, end of key encountered",
                            k, v.pekzep_hanzi
                        ));
                    };

                    if let Some(a) = contraction_pronunciation.iter().find(|(h, syllable)| {
                        **h == contraction
                            && read::phrase::ExtSyllable::Syllable(*syllable) == expected_syllable
                    }) {
                        info!("matched {} with {}", a.0, a.1);
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
                            info!("matched `xizi`.");
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
                        info!("matched {} with {}", a.0, a.1);
                    } else {
                        return Err(format!(
                            "While trying to match {:?} with {}, cannot find the pronunciation `{}` for character `{}`", k, v.pekzep_hanzi,
                            expected_syllable, c
                        ));
                    }
                }
            }

            if let Some(a) = key_iter.next() {
                return Err(format!(
                    "Encountered {} but `{}` ended earlier.\n\nThis occurred while trying to match {:?} with {}, ",
                    a, v.pekzep_hanzi, k, v.pekzep_hanzi,
                ));
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
        contraction_pronunciation: &[(String, pekzep_syllable::PekZepSyllable)],
    ) -> Result<(), String> {
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
                            .find(|(h, sy)| *h == contraction && *sy == syllable)
                        {
                            info!("matched {} with {}", a.0, a.1);
                        } else {
                            return Err(format!(
                            "While trying to match {} with {}, cannot find the contracted pronunciation `{}` for `«{}»`", syllable, contraction,
                            syllable, c
                        ));
                        }
                    } else if let Some(a) = char_pronunciation
                        .iter()
                        .find(|(h, sy)| **h == c.to_string() && *sy == syllable)
                    {
                        info!("matched {} with {}", a.0, a.1);
                    } else {
                        return Err(format!(
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

    fn check_kan1(pekzep_hanzi: &str, english: &str) {
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
    pub fn new() -> Result<Self, Box<dyn Error>> {
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

        let rows3 = collect_any_errors(
            spoonfed_rows
                .iter()
                .map(|(syllables, row)| {
                    match parse_decomposed(&vocab, row).map_err(|e| e.join("\n")) {
                        Ok(decomposition) => {
                            for DecompositionItem {
                                key,
                                voc,
                                splittable_compound_info: _,
                            } in &decomposition
                            {
                                if !vocab_ordered.contains_key(&VocabInternalKey::new(key)) {
                                    vocab_ordered.insert(VocabInternalKey::new(key), voc.clone());
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
            if !vocab_ordered.contains_key(&VocabInternalKey::new(key)) {
                warn!("Item with internal key `{}` is never used", key);
            }
        }

        Ok(Self {
            rows3,
            vocab_ordered,
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]

pub enum SplittableCompoundInfo {
    FormerHalfHash,
    LatterHalfExclamation,
}

#[derive(Debug, Clone)]
pub struct DecompositionItem {
    pub key: String,
    pub voc: read::vocab::Item,
    pub splittable_compound_info: Option<SplittableCompoundInfo>,
}

/// Checks if:
/// * all the morphemes listed in `row.decomposed` are in the vocab list
/// * the `row.decomposed` really is a decomposition of `row.pekzep_hanzi`.
fn parse_decomposed(
    vocab: &HashMap<String, read::vocab::Item>,
    row: &read::phrase::Item,
) -> Result<Vec<DecompositionItem>, Vec<String>> {
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
                    let splittable_compound_info = if a.contains('!') {
                        Some(SplittableCompoundInfo::LatterHalfExclamation)
                    } else if a.contains('#') {
                        Some(SplittableCompoundInfo::FormerHalfHash)
                    } else {
                        None
                    };
                    Ok(DecompositionItem {
                        key,
                        voc: res?.clone(),
                        splittable_compound_info,
                    })
                })
                .collect::<Vec<_>>(),
        )
    }
}
