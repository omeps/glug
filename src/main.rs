use log::Level;

fn main() {
    for _ in 0..termsize::get().unwrap().cols {
        print!("-");
    }
    print!("\n");
    let (writer, logger) = glug::GLogger::setup();
    let mut l = 2;
    for i in 0..1000 {
        l = (l + 1) % 5;
        log::log!(
            match l {
                0 => Level::Warn,
                1 => Level::Trace,
                2 => Level::Debug,
                3 => Level::Error,
                _ => Level::Info,
            },
            "logged a message"
        );
        std::thread::sleep(std::time::Duration::from_millis(7))
    }
    logger.end();
    writer.join().unwrap();
}
