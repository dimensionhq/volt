#[path = "../classes/init_data.rs"]
mod init_data;

use crate::prompt::prompt::{Confirm, Input, Select};
use colored::Colorize;
use init_data::InitData;
use std::fs::File;
use std::io::Write;
use std::{env, process};

pub fn init(flags: &Vec<String>) {
    let temp = env::current_dir().unwrap().to_str().unwrap().to_string();
    let split: Vec<&str> = temp.split(r"\").collect::<Vec<&str>>();
    let cwd: String = split[split.len() - 1].to_string();

    if flags.contains("-y") || flags.contains("--yes") {

    }

    // Get "name"
    let input: Input = Input {
        message: String::from("name"),
        default: Some(cwd),
        allow_empty: false,
    };

    let name = input.run().unwrap();

    // Get "version"
    let input: Input = Input {
        message: String::from("version"),
        default: Some(String::from("1.0.0")),
        allow_empty: false,
    };

    let version = input.run().unwrap();

    // Get "description"
    let input: Input = Input {
        message: String::from("description"),
        default: None,
        allow_empty: true,
    };

    let description = input.run().unwrap_or_else(|error| {
        // Handle Error
        eprintln!("{}", error);
        process::exit(1);
    });

    // Get "main"
    let input: Input = Input {
        message: String::from("main"),
        default: Some(String::from("index.js")),
        allow_empty: false,
    };

    let main = input.run().unwrap();

    // Get "author"
    let input: Input = Input {
        message: String::from("author"),
        default: None,
        allow_empty: true,
    };

    let author = input.run().unwrap();

    // Get "repository"
    let input: Input = Input {
        message: String::from("repository"),
        default: None,
        allow_empty: true,
    };

    let repository = input.run().unwrap();

    let licenses: Vec<String> = vec![
        String::from("MIT License"),
        String::from("Apache License 2.0"),
        String::from("BSD 3-Clause \"New\" or \"Revised\" License"),
        String::from("BSD 2-Clause \"Simplified\" or \"FreeBSD\" License"),
        String::from("GNU General Public License (GPL)"),
        String::from("GNU Library or \"Lesser\" General Public License (LGPL)"),
        String::from("Mozilla Public License 2.0"),
        String::from("Common Development and Distribution License"),
        String::from("The Unlicense"),
        String::from("Other"),
    ];

    let select = Select {
        message: String::from("License"),
        paged: true,
        selected: Some(1),
        items: licenses.clone(),
    };

    select.run().unwrap();

    let license = &licenses[select.selected.unwrap()];

    let input = Confirm {
        message: String::from("private"),
        default: false,
    };

    let private = input.run().unwrap();

    let data = InitData {
        name: name,
        version: version,
        description: description,
        main: main,
        repository: repository,
        author: author,
        license: license.clone(),
        private: private,
    };

    let mut file = File::create(r"package.json").unwrap();
    file.write(data.dump().as_bytes()).unwrap_or_else(|error| {
        eprintln!(
            "{} : {}",
            "Failed To Create package.json".bright_red(),
            error.to_string().bright_yellow().bold()
        );
        process::exit(1);
    });
}
