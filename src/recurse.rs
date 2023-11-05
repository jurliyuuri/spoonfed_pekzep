pub fn foo() -> String {
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
}

pub fn indent(depth: usize, input: &str) -> String {
    input
        .lines()
        .map(|s| format!("{}{s}", "    ".repeat(depth)))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn a(c: char) -> String {
    format!(
        r#"<a href="./{c}.html" style="text-decoration: none;">
    <div class="dismantling-bar">
        <span style="font-family: LinzklarRounded;">{c}</span><span lang="ja">【{c}】</span>
    </div>
</a>"#
    )
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

pub fn foo2() -> String {
    let bar = format!("{}\n{}", &a('奮'), binary(&a('心'), &a('火')),);

    indent(4, &format!("{}\n{}", a('酒'), binary(&bar, &a('水')),))
}

#[test]
fn test_foo() {
    assert_eq!(foo(), foo2());
}
