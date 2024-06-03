use std::fmt::Display;

macro_rules! set_cursor {
    ($l:expr,$c:expr) => {
        format!("\x1b[{};{}H", $l, $c)
    };
    () => {
        "\x1b[H"
    };
}
macro_rules! clear_to_eol {
    () => {
        "\x1b[0J"
    };
}
macro_rules! color {
    ($c:expr) => {
        format!("\x1b[{}m", $c)
    };
}
#[repr(usize)]
///the numbers for the basic ANSI colors set by the terminal. These are the **widely used** colors for
///the terminal, not specific ones. They **may not be the right color** as they are terminal
///dependent.
pub enum Ansi8 {
    Black = 30,
    Red = 31,
    Green = 32,
    Yellow = 33,
    Blue = 34,
    Magenta = 35,
    Cyan = 36,
    White = 37,
    ///not to be confused with Default
    Default = 39,
    ///not to be confused with Default
    Reset = 0,
}
use Ansi8::*;
pub fn background_match(col: Ansi8) -> usize {
    match col {
        Reset => 0,
        _ => col as usize + 10,
    }
}
pub(crate) use {clear_to_eol, color, set_cursor};
