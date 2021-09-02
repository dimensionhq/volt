use colored::{ColoredString, Colorize};
use std::fmt::Display;

pub trait CustomColorize: Colorize {
    fn caused_by_style(self) -> ColoredString
    where
        Self: Sized,
    {
        self.italic().truecolor(190, 190, 190)
    }

    fn error_style(self) -> ColoredString
    where
        Self: Sized,
    {
        self.on_bright_red().black()
    }

    fn warning_style(self) -> ColoredString
    where
        Self: Sized,
    {
        self.bright_yellow().bold()
    }

    fn info_style(self) -> ColoredString
    where
        Self: Sized,
    {
        self.bright_purple().bold()
    }

    fn success_style(self) -> ColoredString
    where
        Self: Sized,
    {
        self.bright_green().bold()
    }
}

impl<T: Colorize> CustomColorize for T {}

pub trait ResultLogErrorExt {
    fn unwrap_and_handle_error(self);
}

impl<E: Display> ResultLogErrorExt for Result<(), E> {
    fn unwrap_and_handle_error(self) {
        if let Err(e) = self {
            println!("{} {}", "error".error_style(), e);
        }
    }
}

#[macro_export]
macro_rules! error {
    ($($tt:tt)*) => { print!("{} ", $crate::utils::helper::CustomColorize::error_style(" ERROR ")); println!($($tt)*); };
}

#[macro_export]
macro_rules! warning {
    ($($tt:tt)*) => { print!("{}", $crate::utils::helper::CustomColorize::warning_style("warning: ")); println!($($tt)*); };
}

#[macro_export]
macro_rules! info {
    ($($tt:tt)*) => { print!("{}", $crate::utils::helper::CustomColorize::info_style("info: ")); println!($($tt)*); };
}
