//!Logger with graphical elements. For use with `log` crate.
//!The logger writes to stderr with ANSI escape codes to display logs and aggregate data.
//!Could be useful if a program or library logs en masse.
//!# How to use
//!The `glug` logger uses a dedicated writer thread. Panicking may break.
//!(yes, this example is everywhere in this doc)
//!```
//!fn main() {
//!    let gref = glug::GLogger::setup();
//!    log::info!("logged a message");
//!}
//!```
mod glogger;
pub use glogger::gstore::GStore;
pub use glogger::options;
pub use glogger::termpin::elements;
pub use glogger::termpin::*;
pub use glogger::Ansi8;
pub use glogger::GLogger;
pub use glogger::GLoggerOptions;
pub use glogger::GLoggerRef;
