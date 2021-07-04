pub fn capitalize_first_char(s: &str) -> ::askama::Result<String> {
    Ok(normalizer::capitalize_first_char(s))
}
pub fn line_breaks_and_tabs(s: &str) -> ::askama::Result<String> {
    Ok(normalizer::line_breaks_and_tabs(s))
}
pub fn normalize_chinese_punctuation(s: &str) -> ::askama::Result<String> {
    Ok(normalizer::normalize_chinese_punctuation(s))
}
pub fn normalize_a_b_dialogue(s: &str) -> ::askama::Result<String> {
    Ok(normalizer::normalize_a_b_dialogue(s))
}

pub mod normalizer {
    pub fn capitalize_first_char(s: &str) -> String {
        let mut v: Vec<char> = s.chars().collect();
        v[0] = v[0].to_uppercase().next().unwrap();
        let s2: String = v.into_iter().collect();
        s2
    }
    pub fn line_breaks_and_tabs(s: &str) -> String {
        let s = s.to_string();
        format!(
            "<table border=\"1\" cellpadding=\"5\" cellspacing=\"0\">\n\t<tr><td>{}</td></tr>\n</table>",
            s.replace("\t", "</td><td>")
                .replace("\n", "</td></tr>\n\t<tr><td>")
        )
    }
    pub fn normalize_chinese_punctuation(s: &str) -> String {
        let s = s.to_string();
        s.replace(',', "，").replace('?', "？").replace('!', "！")
    }
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
}
