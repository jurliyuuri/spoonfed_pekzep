#[must_use]
/// Capitalizes the first character of a string.
/// ```
/// use spoonfed_pekzep::normalizer::capitalize_first_char;
/// assert_eq!(capitalize_first_char("ā, wǒ rènshi zhège nánrén!"), "Ā, wǒ rènshi zhège nánrén!");
/// ```
pub fn capitalize_first_char(text: &str) -> String {
    let mut iter = text.chars();
    iter.next().map_or_else(String::new, |init| {
        format!("{}{}", init.to_uppercase(), iter.collect::<String>())
    })
}

#[must_use]
pub fn convert_line_breaks_and_tabs_into_single_table(s: &str) -> String {
    format!(
        "<table border=\"1\" cellpadding=\"5\" cellspacing=\"0\">\n{}</table>",
        convert_line_breaks_and_tabs_into_trs_and_tds(s)
    )
}

#[must_use]
pub fn convert_line_breaks_and_tabs_into_trs_and_tds(s: &str) -> String {
    let s = s.to_string();
    format!(
        "\t<tr><td>{}</td></tr>\n",
        s.replace('\t', "</td><td>")
            .replace('\n', "</td></tr>\n\t<tr><td>")
    )
}

#[must_use]
pub fn convert_into_multiple_tables(s: &str) -> String {
    s.split("\n\n")
        .map(convert_line_breaks_and_tabs_into_single_table)
        .collect::<Vec<_>>()
        .join("<br>")
}

#[must_use]
pub fn normalize_chinese_punctuation(s: &str) -> String {
    let s = s.to_string();
    s.replace(',', "，").replace('?', "？").replace('!', "！")
}

#[must_use]
pub fn normalize_a_b_dialogue(s: &str) -> String {
    if s.starts_with('A') && s.contains('B') {
        format!(
            "「{}」",
            &s[1..]
                .replace(" B", "」「")
                .replace('B', "」「")
                .replace(" A", "」「")
                .replace('A', "」「")
        )
    } else {
        s.to_string()
    }
}
