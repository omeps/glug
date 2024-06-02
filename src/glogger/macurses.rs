use std::fmt::Display;

#[macro_export]
macro_rules! set_cursor {
    ($l:expr,$c:expr) => {
        format!("\x1b[{};{}H", $l, $c)
    };
    () => {
        "\x1b[H"
    };
}
#[macro_export]
macro_rules! clear_to_eol {
    () => {
        "\x1b[0J"
    };
}
#[macro_export]
macro_rules! color {
    ($c:expr) => {
        format!("\x1b[{}m", $c)
    };
}
#[repr(usize)]
#[derive(Debug, Clone, Copy)]
pub enum Ansi8 {
    Black = 30,
    Red = 31,
    Green = 32,
    Yellow = 33,
    Blue = 34,
    Magenta = 35,
    Cyan = 36,
    White = 37,
    Default = 39,
    Reset = 0,
}
