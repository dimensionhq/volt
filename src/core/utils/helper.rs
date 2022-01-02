/*
    Copyright 2021 Volt Contributors

    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at

        http://www.apache.org/licenses/LICENSE-2.0

    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.
*/

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

macro_rules! error {
    ($($tt:tt)*) => { print!("{} ", $crate::core::utils::helper::CustomColorize::error_style(" ERROR ")); println!($($tt)*); };
}

macro_rules! warning {
    ($($tt:tt)*) => { print!("{}", $crate::core::utils::helper::CustomColorize::warning_style("warning: ")); println!($($tt)*); };
}

macro_rules! info {
    ($($tt:tt)*) => { print!("{}", $crate::core::utils::helper::CustomColorize::info_style("info: ")); println!($($tt)*); };
}
