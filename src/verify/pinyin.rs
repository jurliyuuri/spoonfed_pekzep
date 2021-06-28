pub fn check(hanzi_str: &str, pinyin_str: &str) {
    use log::warn;
    use pinyin::ToPinyinMulti;
    use pinyin_parser::PinyinParser;

    // first do a fast, naive check
    let mut pinyins = vec![];
    for multi in hanzi_str.to_pinyin_multi().flatten() {
        let mut candidates = vec![];
        for p in multi {
            candidates.push(p.with_tone());
            candidates.push(p.plain()); // to allow light tone
        }
        pinyins.push(candidates);
    }

    println!("parsing {:?}:", pinyin_str);

    let pinyins2 = PinyinParser::strict(pinyin_str)
        .into_iter()
        .collect::<Vec<_>>();

    if pinyins2.len() != pinyins.len() {
        warn!(
            "length differs:\npinyins: {:?}\npinyins2: {:?}",
            pinyins, pinyins2
        );
    }

    for (i, p) in pinyins2.iter().enumerate() {
        if pinyins[i].contains(&&p[..]) {
            continue;
        }

        warn!("{} not found within candidates {:?}", p, pinyins[i]);
    }
}
