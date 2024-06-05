use log::Level::*;
use rand::Rng;
fn main() {
    let mut rng = rand::thread_rng();
    let (writer, logger) = glug::GLogger::setup();
    for _ in 0..250 {
        log::log!(
            match rng.gen_range(0..5) {
                0 => Trace,
                1 => Debug,
                2 => Info,
                3 => Warn,
                _ => Error,
            },
            "log message"
        );
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    logger.end();
    writer.join().unwrap();
}
