fn main() {
    let _gref = glug::GLogger::setup();
    for i in 0..10 {
        log::warn!("message");
    }
    log::error!("");
    panic!();
}
