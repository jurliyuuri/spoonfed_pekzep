mod filters;
use askama::Template;

#[derive(Template)]
#[template(path = "phrase.html")]
pub struct PhraseTemplate<'a> {
    pub english: &'a str,
    pub chinese_hanzi: &'a str,
    pub chinese_pinyin: &'a str,
    pub pekzep_latin: &'a str,
    pub pekzep_hanzi: &'a str,
    pub japanese: &'a str,
    pub prev_link: &'a str,
    pub next_link: &'a str,
    pub wav_tag: &'a str,
    pub analysis: &'a str,
    pub oga_tag: &'a str,
    pub pekzep_images: &'a str,
    pub author_color: &'a str,
    pub author_name: &'a str,
    pub has_audio: bool,
    pub has_japanese: bool,
}

#[derive(Template)]
#[template(path = "ind.html")]
pub struct IndTemplate<'a> {
    pub index: &'a str,
    pub length: usize,
    pub how_many_glosses: usize,
}

#[derive(Template)]
#[template(path = "vocab.html")]
pub struct VocabTemplate<'a> {
    pub analysis: &'a str,
    pub usage_table: &'a str,
}

#[derive(Template)]
#[template(path = "vocab_list.html")]
pub struct VocabListTemplate<'a> {
    pub vocab_html: &'a str,
}

#[derive(Template)]
#[template(path = "vocab_list_internal.html")]
pub struct VocabListInternalTemplate<'a> {
    pub vocab_html: &'a str,
    pub header_row: &'a str,
}

#[derive(Template)]
#[template(path = "char_list.html")]
pub struct CharListTemplate<'a> {
    pub char_list_table: &'a str,
}

#[derive(Template)]
#[template(path = "char.html")]
pub struct CharTemplate<'a> {
    pub title_img: &'a str,
    pub transcription_char: &'a str,
    pub pronunciations: &'a str,
    pub occurrence_count: &'a str,
    pub variants: &'a str,
    pub variant_of: &'a str,
    pub summary_occurrence_list: &'a str,
    pub word_table: &'a str,
    pub dismantling: &'a str,
}
