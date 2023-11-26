use crate::read::char_pronunciation::Linzklar;
use crate::read::linzklar_dismantling::{
    self, CustomUnaryOperator, DismantlingTree, IdsBinaryOperator, IdsTrinaryOperator,
};
use crate::recurse::{foo3, indent, Subtree, Tree};
use askama::Template;

use crate::askama_templates::CharTemplate;
use crate::read;
use crate::{convert_hanzi_to_images, convert_hanzi_to_images_with_size, verify};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;

fn construct_tree_from_dismantlingtree(
    parsed_dismantle: &HashMap<Linzklar, DismantlingTree>,
    dismantling_tree: &DismantlingTree,
) -> Tree<char> {
    match dismantling_tree {
        DismantlingTree::Leaf(c) => construct_tree_from_linzklar(parsed_dismantle, *c), /* do another lookup */
        DismantlingTree::Binary(IdsBinaryOperator::Unit, d1, d2) => Tree::MaybeLabelledSubTree {
            label: None,
            subtree: Subtree::Binary(
                Box::new(construct_tree_from_dismantlingtree(parsed_dismantle, d1)),
                Box::new(construct_tree_from_dismantlingtree(parsed_dismantle, d2)),
            ),
        },
        DismantlingTree::Trinary(IdsTrinaryOperator::Unit, d1, d2, d3) => {
            Tree::MaybeLabelledSubTree {
                label: None,
                subtree: Subtree::Trinary(
                    Box::new(construct_tree_from_dismantlingtree(parsed_dismantle, d1)),
                    Box::new(construct_tree_from_dismantlingtree(parsed_dismantle, d2)),
                    Box::new(construct_tree_from_dismantlingtree(parsed_dismantle, d3)),
                ),
            }
        }
        DismantlingTree::Unary(CustomUnaryOperator::Explosion, d1) => Tree::MaybeLabelledSubTree {
            label: None,
            subtree: Subtree::Binary(
                Box::new(construct_tree_from_dismantlingtree(parsed_dismantle, d1)),
                Box::new(Tree::Leaf { label: '💥' }),
            ),
        },
        DismantlingTree::Unary(CustomUnaryOperator::Rotation, d1) => Tree::MaybeLabelledSubTree {
            label: None,
            subtree: Subtree::Binary(
                Box::new(construct_tree_from_dismantlingtree(parsed_dismantle, d1)),
                Box::new(Tree::Leaf { label: '↺' }),
            ),
        },
    }
}

fn construct_tree_from_linzklar(
    parsed_dismantle: &HashMap<Linzklar, DismantlingTree>,
    linzklar: Linzklar,
) -> Tree<char> {
    let Some(dismantling_tree) = parsed_dismantle.get(&linzklar) else {
        return Tree::Leaf {
            label: linzklar.as_char(),
        };
    };
    match dismantling_tree {
        DismantlingTree::Leaf(_) => Tree::Leaf {
            label: linzklar.as_char(),
        },
        DismantlingTree::Binary(IdsBinaryOperator::Unit, d1, d2) => Tree::MaybeLabelledSubTree {
            label: Some(linzklar.as_char()),
            subtree: Subtree::Binary(
                Box::new(construct_tree_from_dismantlingtree(parsed_dismantle, d1)),
                Box::new(construct_tree_from_dismantlingtree(parsed_dismantle, d2)),
            ),
        },
        DismantlingTree::Trinary(IdsTrinaryOperator::Unit, d1, d2, d3) => {
            Tree::MaybeLabelledSubTree {
                label: Some(linzklar.as_char()),
                subtree: Subtree::Trinary(
                    Box::new(construct_tree_from_dismantlingtree(parsed_dismantle, d1)),
                    Box::new(construct_tree_from_dismantlingtree(parsed_dismantle, d2)),
                    Box::new(construct_tree_from_dismantlingtree(parsed_dismantle, d3)),
                ),
            }
        }
        DismantlingTree::Unary(CustomUnaryOperator::Explosion, d1) => Tree::MaybeLabelledSubTree {
            label: Some(linzklar.as_char()),
            subtree: Subtree::Binary(
                Box::new(construct_tree_from_dismantlingtree(parsed_dismantle, d1)),
                Box::new(Tree::Leaf { label: '💥' }),
            ),
        },
        DismantlingTree::Unary(CustomUnaryOperator::Rotation, d1) => Tree::MaybeLabelledSubTree {
            label: Some(linzklar.as_char()),
            subtree: Subtree::Binary(
                Box::new(construct_tree_from_dismantlingtree(parsed_dismantle, d1)),
                Box::new(Tree::Leaf { label: '↺' }),
            ),
        },
    }
}

fn get_occurrence_list(data_bundle: &verify::DataBundle, linzklar: Linzklar) -> Vec<String> {
    let mut occurrence_list = vec![];
    for verify::Rows3Item {
        syllables,
        decomposition: _,
        row,
    } in &data_bundle.rows3
    {
        if row.pekzep_hanzi.contains(&format!("{linzklar}")) {
            occurrence_list.push(format!(
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
            ));
        }
    }
    occurrence_list
}

fn get_word_table(data_bundle: &verify::DataBundle, linzklar: Linzklar, rel_path: &str) -> Vec<String> {
    let mut word_table = vec![];
    for (key, vocab) in &data_bundle.vocab_ordered {
        if vocab.pekzep_hanzi.contains(linzklar.as_char()) {
            let link_path = format!("{rel_path}/vocab/{}.html", key.to_path_safe_string());
            let rel_path = "..";
            word_table.push(format!(
                "<a href=\"{link_path}\">{}</a>\t{}\t<span style=\"filter:brightness(65%) contrast(500%);\">{}</span>\t{}\t{}\t{}",
                vocab.pekzep_latin,
                vocab.pekzep_hanzi,
                convert_hanzi_to_images(&vocab.pekzep_hanzi, "/{} N()SL«»", rel_path) ,
                vocab.parts_of_speech,
                vocab.parts_of_speech_supplement,
                vocab.english_gloss
            ));
        }
    }
    word_table
}

/// Generates `char/`
/// # Errors
/// Will return `Err` if the file I/O fails or the render panics.
pub fn gen(data_bundle: &verify::DataBundle) -> Result<(), Box<dyn Error>> {
    let parsed_dismantle = linzklar_dismantling::parse()?;

    let rel_path = "..";
    let (char_pronunciation, variants_to_standard) = read::char_pronunciation::parse()?;

    let extended_char_count = char_pronunciation
        .iter()
        .map(|(lin, _)| (*lin, *data_bundle.char_count.get(lin).unwrap_or(&0)))
        .collect::<HashMap<_, _>>();

    for (linzklar, count) in &extended_char_count {
        let mut file = File::create(format!("docs/char/{linzklar}.html"))?;

        let mut variants = variants_to_standard
            .iter()
            .filter_map(|(key, value)| if value == linzklar { Some(key) } else { None })
            .collect::<Vec<_>>();
        variants.sort(); // ソートしておくことで、毎ビルドごとに HTML の差分が出るのを避ける

        let word_table = get_word_table(data_bundle, *linzklar, rel_path);
        let occurrence_list = get_occurrence_list(data_bundle, *linzklar);

        let summary_occurrence_list = if occurrence_list.is_empty() {
            String::new()
        } else {
            format!(
                r##"<details>
            <summary style="font-size: 80%; font-weight: bold; margin: -0.5em -0.5em 0; padding: 0.5em;"><span lang="en">Show all occurrences</span> / <span lang="zh-CN">显示所有例句</span> / <span lang="ja">全ての出現例を表示</span></summary>
        {}
    </details>"##,
                occurrence_list.join("\n")
            )
        };

        let variants = if variants.is_empty() {
            String::new()
        } else {
            format!(
                r#"<hr>
<p><span lang="en">variants</span> / <span lang="zh-CN">异体字</span> / <span lang="ja">異体字</span>
<ul>
{}
</ul>
</p>"#, variants
            .iter()
            .map(|variant| format!(
                r#"            <li><span style="filter:brightness(65%) contrast(500%);">{}</span>【{variant}】</li>"#,
                convert_hanzi_to_images(&format!("{variant}"), "/{} N()SL«»", rel_path)
            ))
            .collect::<Vec<_>>()
            .join("\n"))
        };

        let variant_of = variants_to_standard.get(linzklar).map_or_else(String::new, |v| {
                let base_char = format!(
                    r#"<span style="filter:brightness(65%) contrast(500%);">{}</span>【{v}】"#,
                    convert_hanzi_to_images(&format!("{v}"), "/{} N()SL«»", rel_path)
                );
                format!(
                    r#"<hr>
    <p><span lang="en">A variant of {base_char}</span> / <span lang="zh-CN">{base_char}的异体字</span> / <span lang="ja">{base_char}の異体字</span></p>"#,
                )
            });

        let dismantling = indent(
            4,
            &foo3(construct_tree_from_linzklar(&parsed_dismantle, *linzklar)),
        );

        write!(
            file,
            "{}",
            CharTemplate {
                title_img: &format!(
                    "<span style=\"filter:brightness(65%) contrast(500%);\">{}</span>",
                    convert_hanzi_to_images_with_size(
                        &format!("{linzklar}"),
                        "/{} N()SL«»",
                        rel_path,
                        130
                    ),
                ),
                transcription_char: &format!("{linzklar}"),
                pronunciations: &char_pronunciation
                    .iter()
                    .filter_map(|(lin, syl)| if lin == linzklar {
                        Some(syl.to_string())
                    } else {
                        None
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
                occurrence_count: &format!("{count}"),
                word_table: &word_table.join("\n"),
                summary_occurrence_list: &summary_occurrence_list,
                variants: &variants,
                variant_of: &variant_of,
                dismantling: &dismantling,
            }
            .render()?
        )?;
    }
    Ok(())
}
