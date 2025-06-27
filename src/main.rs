use spoonfed_pekzep::*;
use std::error::Error;

fn reset_folder(path: &str) -> Result<(), Box<dyn Error>> {
    eprintln!("Resetting {}", path);
    std::fs::remove_dir_all(path)?;
    std::fs::create_dir(path)?;
    Ok(())
}
fn main() -> Result<(), Box<dyn Error>> {
    use std::env;
    if env::var("RUST_LOG").is_err() {
        // TODO: Audit that the environment access only happens in single-threaded code.
        unsafe { env::set_var("RUST_LOG", "warn") };
    }
    env_logger::init();

    // At each run, reset the content of docs/char_img,
    // because later in the script this folder is automatically filled
    // by the images required to render the page.
    reset_folder("docs/char_img")?;

    // At each run, reset the content of docs/phrase, docs/vocab and docs/char
    // because they will be filled later in the script.
    reset_folder("docs/phrase")?;
    reset_folder("docs/char")?;
    reset_folder("docs/vocab")?;

    let data_bundle = verify::DataBundle::new()?;

    eprintln!("Generating docs/phrase/");
    generate::phrases::r#gen(&data_bundle)?;

    eprintln!("Generating docs/vocab/");
    generate::vocabs::r#gen(&data_bundle)?;

    eprintln!("Generating docs/vocab_list_internal.html");
    generate::vocab_list_internal::r#gen(&data_bundle)?;

    eprintln!("Generating docs/vocab_list.html");
    generate_vocab_list(&data_bundle)?;

    eprintln!("Generating docs/char/");
    generate::chars::r#gen(&data_bundle)?;

    eprintln!("Generating docs/char_list.html");
    generate_char_list(&data_bundle)?;

    eprintln!("Generating docs/index.html");
    generate_index(&data_bundle)?;

    eprintln!("Writing docs/raw.tsv");
    write_condensed_csv()?;

    eprintln!("Writing docs/raw.js");
    write_condensed_js()?;

    eprintln!("Writing docs/char_count.js");
    write_char_count_js(&data_bundle.char_count)?;

    Ok(())
}
