fn main() {
    let (writer, logger) = glug::GLogger::setup();
    log::info!("logged a message");
    log::trace!("logged a message");
    log::warn!("logged a message");
    log::error!("logged a message");
    log::debug!("logged a message");
    logger.end();
    writer.join().unwrap();
}
