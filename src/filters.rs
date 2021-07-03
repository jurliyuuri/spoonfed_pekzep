pub fn capitalize_first_char(s: &str) -> ::askama::Result<String> {
    Ok(crate::normalizer::capitalize_first_char(s))
}
pub fn line_breaks_and_tabs(s: &str) -> ::askama::Result<String> {
    Ok(crate::normalizer::line_breaks_and_tabs(s))
}
pub fn normalize_chinese_punctuation(s: &str) -> ::askama::Result<String> {
    Ok(crate::normalizer::normalize_chinese_punctuation(s))
}
pub fn normalize_a_b_dialogue(s: &str) -> ::askama::Result<String> {
    Ok(crate::normalizer::normalize_a_b_dialogue(s))
}
