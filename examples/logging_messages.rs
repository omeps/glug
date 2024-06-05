fn main() {
    let (writer, logger) = glug::GLogger::setup();
    log::info!("logged a message");
    logger.end();
    writer.join().unwrap();
}
