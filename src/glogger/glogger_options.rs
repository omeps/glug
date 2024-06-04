use super::macurses::{Ansi8, Ansi8::*};
#[derive(Copy, Clone)]
pub struct GLoggerOptions {
    pub colors: [Ansi8; 5],
}
impl std::default::Default for GLoggerOptions {
    fn default() -> Self {
        Self {
            colors: [Red, Yellow, Green, Blue, Default],
        }
    }
}
