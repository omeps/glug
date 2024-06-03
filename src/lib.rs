//!Logger with graphical elements. For use with `log` crate.
//!The logger writes to stderr with ANSI escape codes to display logs and aggregate data.
//!Could be useful if a program or library logs en masse.
//!# How to use
//!The `glug` logger uses a dedicated writer thread. This thread runs until told to stop. To stop
//!the logger properly, it is a good idea to use `GLogger::end` and `JoinHandle::join` like so:
//!(yes, this example is everywhere in this doc)
//!```
//!fn main() {
//!    let (writer, logger) = glug::GLogger::setup();
//!    log::info!("logged a message");
//!    logger.end();
//!    writer.join().unwrap();
//!}
//!```
mod glogger;
pub use glogger::Ansi8;
pub use glogger::GLogger;
pub use glogger::GLoggerOptions;
