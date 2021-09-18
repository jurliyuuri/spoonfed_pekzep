#[must_use]
pub fn capitalize_first_char(text: &str) -> String {
    let mut iter = text.chars();
    if let Some(init) = iter.next() {
        format!("{}{}", init.to_uppercase(), iter.collect::<String>())
    } else {
        String::new()
    }
}

#[must_use]
pub fn line_breaks_and_tabs(s: &str) -> String {
    let s = s.to_string();
    format!(
        "<table border=\"1\" cellpadding=\"5\" cellspacing=\"0\">\n\t<tr><td>{}</td></tr>\n</table>",
        s.replace("\t", "</td><td>")
            .replace("\n", "</td></tr>\n\t<tr><td>")
    )
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