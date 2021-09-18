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
    pub prev_link: &'a str,
    pub next_link: &'a str,
    pub wav_tag: &'a str,
    pub analysis: &'a str,
    pub oga_tag: &'a str,
    pub pekzep_images: &'a str,
    pub author_color: &'a str,
    pub author_name: &'a str,
    pub has_audio: bool,
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
