#[test]
fn test_foo() {
    fn foo2() -> String {
        let tree = Tree::MaybeLabelledSubTree {
            label: Some('酒'),
            subtree: Subtree::Binary(
                Box::new(Tree::MaybeLabelledSubTree {
                    label: Some('奮'),
                    subtree: Subtree::Binary(
                        Box::new(Tree::Leaf { label: '心' }),
                        Box::new(Tree::Leaf { label: '火' }),
                    ),
                }),
                Box::new(Tree::Leaf { label: '水' }),
            ),
        };

        indent(4, &foo3(tree))
    }
    assert_eq!(
        foo2(),
        String::from(
            r#"                <a href="./酒.html" style="text-decoration: none;">
                    <div class="dismantling-bar">
                        <span style="font-family: LinzklarRounded;">酒</span><span lang="ja">【酒】</span>
                    </div>
                </a>
                <div class="dismantling-frame">
                    <div style="flex: 50%">
                        <a href="./奮.html" style="text-decoration: none;">
                            <div class="dismantling-bar">
                                <span style="font-family: LinzklarRounded;">奮</span><span lang="ja">【奮】</span>
                            </div>
                        </a>
                        <div class="dismantling-frame">
                            <div style="flex: 50%">
                                <a href="./心.html" style="text-decoration: none;">
                                    <div class="dismantling-bar">
                                        <span style="font-family: LinzklarRounded;">心</span><span lang="ja">【心】</span>
                                    </div>
                                </a>
                            </div>
                            <div style="flex: 50%">
                                <a href="./火.html" style="text-decoration: none;">
                                    <div class="dismantling-bar">
                                        <span style="font-family: LinzklarRounded;">火</span><span lang="ja">【火】</span>
                                    </div>
                                </a>
                            </div>
                        </div>
                    </div>
                    <div style="flex: 50%">
                        <a href="./水.html" style="text-decoration: none;">
                            <div class="dismantling-bar">
                                <span style="font-family: LinzklarRounded;">水</span><span lang="ja">【水】</span>
                            </div>
                        </a>
                    </div>
                </div>"#,
        )
    );
}

pub fn indent(depth: usize, input: &str) -> String {
    input
        .lines()
        .map(|s| format!("{}{s}", "    ".repeat(depth)))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn a(c: char) -> String {
    if c == '💥' || c == '↺' {
        format!(
            r#"
    <div class="dismantling-bar">
        <span lang="ja">{c}</span>
    </div>
"#
        )
    } else {
        format!(
            r#"<a href="./{c}.html" style="text-decoration: none;">
    <div class="dismantling-bar">
        <img src="../char_bitmap_img/{c}.png" height="16" /><span lang="ja">【{c}】</span>
    </div>
</a>"#
        )
    }
}

pub fn binary(s1: &str, s2: &str) -> String {
    format!(
        r#"<div class="dismantling-frame">
    <div style="flex: 50%">
{}
    </div>
    <div style="flex: 50%">
{}
    </div>
</div>"#,
        indent(2, s1),
        indent(2, s2)
    )
}

pub fn trinary(s1: &str, s2: &str, s3: &str) -> String {
    format!(
        r#"<div class="dismantling-frame">
    <div style="flex: 33.3333333%">
{}
    </div>
    <div style="flex: 33.3333333%">
{}
    </div>
    <div style="flex: 33.3333333%">
{}
    </div>
</div>"#,
        indent(2, s1),
        indent(2, s2),
        indent(2, s3)
    )
}

pub enum Tree<T> {
    Leaf {
        label: T,
    },
    MaybeLabelledSubTree {
        label: Option<T>,
        subtree: Subtree<T>,
    },
}

pub enum Subtree<T> {
    Binary(Box<Tree<T>>, Box<Tree<T>>),
    Trinary(Box<Tree<T>>, Box<Tree<T>>, Box<Tree<T>>),
}

pub fn foo3(t: Tree<char>) -> String {
    match t {
        Tree::Leaf { label } => a(label),
        Tree::MaybeLabelledSubTree { label, subtree } => {
            format!(
                "{}\n{}",
                label.map_or(String::new(), a),
                match subtree {
                    Subtree::Binary(t1, t2) => binary(&foo3(*t1), &foo3(*t2)),
                    Subtree::Trinary(t1, t2, t3) => trinary(&foo3(*t1), &foo3(*t2), &foo3(*t3)),
                }
            )
        }
    }
}
