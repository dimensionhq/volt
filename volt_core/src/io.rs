#![allow(dead_code)]

use colored::Colorize;

use crate::classes::meta::Meta;

pub fn write(text: String, metadata: Meta) {
    if !metadata.silent {
        if !metadata.no_color {
            println!("{}", text);
        } else {
            println!("{}", text.bright_white());
        }
    }
}

pub fn write_verbose(text: String, metadata: Meta) {
    if !metadata.silent && metadata.verbose {
        if !metadata.no_color {
            println!(
                "{}: {}",
                "verbose".bright_green().bold(),
                text.bright_white()
            );
        } else {
            println!(
                "{}: {}",
                "verbose".bright_white().bold(),
                text.bright_white()
            );
        }
    }
}

pub fn write_debug(text: String, metadata: Meta) {
    if !metadata.silent && metadata.debug {
        if !metadata.no_color {
            println!(
                "{}: {}",
                "debug".bright_yellow().bold(),
                text.bright_white()
            );
        } else {
            println!("{}: {}", "debug".bright_white().bold(), text.bright_white());
        }
    }
}
