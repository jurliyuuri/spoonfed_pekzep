use spoonfed_pekzep::*;
use std::error::Error;
fn main() -> Result<(), Box<dyn Error>> {
    use std::env;
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "warn");
    }
    env_logger::init();

    let data_bundle = verify::DataBundle::new()?;

    eprintln!("Generating phrase/");
    generate_phrases(&data_bundle)?;

    eprintln!("Generating vocab/");
    generate_vocab(&data_bundle)?;

    eprintln!("Generating vocab_list_internal.html");
    generate_vocab_list_internal(&data_bundle)?;

    eprintln!("Generating vocab_list.html");
    generate_vocab_list(&data_bundle)?;

    eprintln!("Generating index.html");
    generate_index(&data_bundle)?;

    eprintln!("Writing raw.tsv");
    write_condensed_csv()?;

    eprintln!("Writing raw.js");
    write_condensed_js()?;

    Ok(())
}
