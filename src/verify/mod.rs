use crate::read;
use linked_hash_map::LinkedHashMap;
use partition_eithers::collect_any_errors;
use std::collections::HashMap;
use std::error::Error;

pub struct Foo {
    pub vocab: HashMap<String, read::vocab::Vocab>,
    pub rows3: Vec<(
        Vec<read::main_row::ExtSyll>,
        Vec<(String, read::vocab::Vocab)>,
        read::main_row::MainRow,
    )>,
    pub vocab_ordered: LinkedHashMap<String, read::vocab::Vocab>,
}

impl Foo {
    pub fn new() -> Result<Foo, Box<dyn Error>> {
        let spoonfed_rows = read::main_row::parse_spoonfed()?;

        let vocab = read::vocab::parse_vocabs()?;
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

        Ok(Foo {
            vocab,
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
