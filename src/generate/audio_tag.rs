use crate::read;

#[must_use]
pub fn generate_oga_tag(
    row: &read::phrase::Item,
    syllables: &[read::phrase::ExtSyllable],
) -> (String, Option<bool>) {
    use log::warn;
    let filename = read::phrase::syllables_to_str_underscore(syllables);
    if row.filetype.contains(&read::phrase::FilePathType::Oga) {
        if !std::path::Path::new(&format!("docs/spoonfed_pekzep_sounds/{filename}.oga")).exists() {
            warn!("oga file not found: {filename}.oga");
        }
        (
            format!(r#"<source src="../spoonfed_pekzep_sounds/{filename}.oga" type="audio/ogg">"#),
            Some(true),
        )
    } else if std::path::Path::new(&format!("docs/spoonfed_pekzep_sounds/{filename}.oga")).exists()
    {
        warn!("oga file IS found, but is not linked: {filename}.oga");
        (String::new(), None)
    } else if std::path::Path::new(&format!("docs/nonreviewed_sounds/{filename}.oga")).exists() {
        (
            format!(r#"<source src="../nonreviewed_sounds/{filename}.oga" type="audio/ogg">"#),
            Some(false),
        )
    } else {
        (String::new(), None)
    }
}

#[must_use]
pub fn generate_wav_tag(
    row: &read::phrase::Item,
    syllables: &[read::phrase::ExtSyllable],
) -> String {
    use log::warn;
    let filename = read::phrase::syllables_to_str_underscore(syllables);
    let wav_file_exists =
        std::path::Path::new(&format!("docs/spoonfed_pekzep_sounds/{filename}.wav")).exists();
    if row.filetype.contains(&read::phrase::FilePathType::Wav) {
        if !wav_file_exists {
            warn!("wav file not found: {}.wav", filename);
        }
        format!(r#"<source src="../spoonfed_pekzep_sounds/{filename}.wav" type="audio/wav">"#)
    } else {
        if wav_file_exists {
            warn!("wav file IS found, but is not linked: {}.wav", filename);
        }
        String::new()
    }
}
