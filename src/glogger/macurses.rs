macro_rules! set_cursor {
    ($l:expr,$c:expr) => {
        format!("\x1b[{};{}H", $l, $c)
    };
    () => {
        "\x1b[H"
    };
}
macro_rules! hide_cursor {
    () => {
        "\x1b[?25l"
    };
}
macro_rules! clear_screen {
    () => {
        "\x1b[2J"
    };
}
macro_rules! show_cursor {
    () => {
        "\x1b[?25h"
    };
}
macro_rules! color {
    ($c:expr) => {
        format!("\x1b[{}m", $c)
    };
}
#[repr(usize)]
#[derive(Copy, Clone, Debug)]
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
pub(crate) use {clear_screen, color, hide_cursor, set_cursor, show_cursor};
