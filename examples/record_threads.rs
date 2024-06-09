use std::thread;

fn main() {
    let (writer, logger) = glug::GLogger::setup_with_options(glug::GLoggerOptions {
        record_threads: Some(glug::options::RecordThreadsOptions {
            seperate_histograms: false,
            summary: false,
        }),
        ..Default::default()
    });
    let other_thread = thread::Builder::new()
        .name("spawned thread".into())
        .spawn(|| log::info!("hello from spawned thread!"))
        .unwrap();
    log::info!("hello from main!");
    other_thread.join().unwrap();
    logger.end();
    writer.join().unwrap();
}
