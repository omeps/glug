use log::Level::*;
use rand::Rng;
fn main() {
    let _gref = glug::GLogger::setup();
    let mut threads = (0..5).map(|_| {
        std::thread::spawn(|| {
            let mut rng = rand::thread_rng();
            for _ in 0..100 {
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
            }
            std::thread::sleep(std::time::Duration::from_millis(25));
        })
    });
    threads.try_for_each(|t| t.join()).unwrap();
}
