pub fn check(hanzi_str: &str, pinyin_str: &str) {
    use log::{info, warn};
    use pinyin::ToPinyinMulti;
    use pinyin_parser::PinyinParser;

    info!("parsing {:?}:", pinyin_str);

    let mut hanzi_iter = hanzi_str.chars();
    for pinyin in PinyinParser::strict(pinyin_str) {
        let (hanzi, candidates) = loop {
            let hanzi = hanzi_iter.next().expect("hanzi ran out");

            if let Some(multi) = hanzi.to_pinyin_multi() {
                let mut candidates = vec![];
                for cand_pinyin in multi {
                    candidates.push(cand_pinyin.with_tone());
                    candidates.push(cand_pinyin.plain()); // to allow light tone
                }
                break (hanzi, candidates);
            }
        };

        if pinyin.ends_with('r') && !["er", "ēr", "ér", "ěr", "èr"].contains(&&pinyin[..]) {
            // Erhua. Get the next Chinese character and verify that it is 儿 or 兒
            loop {
                let expect_儿 = hanzi_iter.next().expect("hanzi ran out, expected 儿 or 兒");
                if expect_儿.to_pinyin_multi().is_some() {
                    if "儿兒".contains(expect_儿) {
                        break;
                    }
                    panic!(
                        "expected 儿 or 兒 because of the rhotic pinyin {}, but instead found a Chinese character {}",
                        pinyin, expect_儿
                    )
                }
            }
        } else {
            if candidates.contains(&&pinyin[..]) {
                continue;
            }

            warn!(
                "{} not found within candidates {:?} possible for the Chinese character {}. Encountered this while matching `{}` with `{}`.",
                pinyin, candidates, hanzi, pinyin_str, hanzi_str
            );
        }
    }
}
