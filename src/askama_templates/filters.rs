use crate::normalizer;

#[allow(clippy::unnecessary_wraps)]
/// # Errors
/// Never fails. This function returns `Result` solely for the purpose of conforming to the askama's interface.
pub fn capitalize_first_char(s: &str) -> ::askama::Result<String> {
    Ok(normalizer::capitalize_first_char(s))
}

#[allow(clippy::unnecessary_wraps)]
/// # Errors
/// Never fails. This function returns `Result` solely for the purpose of conforming to the askama's interface.
pub fn convert_line_breaks_and_tabs_into_single_table(s: &str) -> ::askama::Result<String> {
    Ok(normalizer::convert_line_breaks_and_tabs_into_single_table(s))
}

#[allow(clippy::unnecessary_wraps)]
/// # Errors
/// Never fails. This function returns `Result` solely for the purpose of conforming to the askama's interface.
pub fn convert_into_multiple_tables(s: &str) -> ::askama::Result<String> {
    Ok(normalizer::convert_into_multiple_tables(s))
}

#[allow(clippy::unnecessary_wraps)]
/// # Errors
/// Never fails. This function returns `Result` solely for the purpose of conforming to the askama's interface.
pub fn normalize_chinese_punctuation(s: &str) -> ::askama::Result<String> {
    Ok(normalizer::normalize_chinese_punctuation(s))
}

#[allow(clippy::unnecessary_wraps)]
/// # Errors
/// Never fails. This function returns `Result` solely for the purpose of conforming to the askama's interface.
pub fn normalize_a_b_dialogue(s: &str) -> ::askama::Result<String> {
    Ok(normalizer::normalize_a_b_dialogue(s))
}
