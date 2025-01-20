fn main() {
    let _gref = glug::GLogger::setup();
    for _ in 0..10 {
        log::warn!("message");
    }
    log::error!("");
    panic!();
}
