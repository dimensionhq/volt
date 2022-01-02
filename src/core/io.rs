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

#![allow(dead_code)]

use crate::core::classes::meta::Meta;

use colored::Colorize;

pub fn write(text: &str, metadata: &Meta) {
    if !metadata.silent {
        if metadata.no_color {
            println!("{}", text.bright_white());
        } else {
            println!("{}", text);
        }
    }
}

pub fn write_verbose(text: &str, metadata: &Meta) {
    if !metadata.silent && metadata.verbose {
        if metadata.no_color {
            println!(
                "{}: {}",
                "verbose".bright_white().bold(),
                text.bright_white()
            );
        } else {
            println!(
                "{}: {}",
                "verbose".bright_green().bold(),
                text.bright_white()
            );
        }
    }
}

pub fn write_debug(text: &str, metadata: &Meta) {
    if !metadata.silent && metadata.debug {
        if metadata.no_color {
            println!("{}: {}", "debug".bright_white().bold(), text.bright_white());
        } else {
            println!(
                "{}: {}",
                "debug".bright_yellow().bold(),
                text.bright_white()
            );
        }
    }
}
