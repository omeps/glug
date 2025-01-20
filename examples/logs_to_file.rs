fn main() {
    let mut path = String::new();
    println!("path: ");
    std::io::stdin().read_line(&mut path).unwrap();
    let _gref = glug::GLogger::setup_with_options(glug::GLoggerOptions {
        save_to_file: Some(path.trim().to_string()),
        ..Default::default()
    });
    log::info!("logged a message");
    log::trace!("logged another message");
}
