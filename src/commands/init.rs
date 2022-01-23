/*
    Copyright 2021, 2022 Volt Contributors

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

use crate::{
    cli::{VoltCommand, VoltConfig},
    core::{
        classes::init_data::{InitData, License},
        prompt::prompts::{Confirm, Input, Select},
        utils,
        utils::errors::VoltError,
        utils::extensions::PathExtensions,
    },
};

use async_trait::async_trait;
use clap::Parser;
use colored::Colorize;
use miette::{IntoDiagnostic, Result};
use regex::Regex;
use std::{fs::File, io::Write, time::Instant};

const PACKAGE_JSON: &str = "package.json";

/// Interactively create or update a package.json file for a project
#[derive(Debug, Parser)]
pub struct Init {
    /// Use default options
    #[clap(short, long)]
    yes: bool,
}

#[async_trait]
impl VoltCommand for Init {
    /// Execute the `volt init` command
    ///
    /// Interactively create or update a package.json file for a project.
    /// ## Arguments
    /// * `app` - Instance of the command (`Arc<App>`)
    /// * `packages` - List of packages to add (`Vec<String>`)
    /// * `flags` - List of flags passed in through the CLI (`Vec<String>`)
    /// ## Examples
    /// ```
    /// // Initialize a new package.json file without any prompts
    /// // .exec() is an async call so you need to await it
    /// Init.exec(app, vec![], vec!["--yes"]).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(self, config: VoltConfig) -> Result<()> {
        let start = Instant::now();

        // get name of cwd
        let cwd_name = config
            .cwd()?
            .file_name_as_string()
            .ok_or(VoltError::GetCurrentDirNameError)?;

        let data = if self.yes {
            // Set name to current directory name
            automatic_initialization(cwd_name, &config)?
        } else {
            manual_initialization(cwd_name, &config)?
        };

        let mut file = File::create(PACKAGE_JSON).map_err(|e| VoltError::WriteFileError {
            source: e,
            name: String::from(PACKAGE_JSON),
        })?;

        file.write(data.into_string().as_bytes())
            .map_err(|e| VoltError::WriteFileError {
                source: e,
                name: String::from(PACKAGE_JSON),
            })?;

        println!("{}", "Successfully Initialized package.json".bright_green());

        Ok(())
    }
}

fn automatic_initialization(name: String, config: &VoltConfig) -> Result<InitData> {
    let version = "0.1.0".to_string();

    let description = None;

    let main = "index.js".to_string();

    let author = {
        let git_user_name = utils::get_git_config(config, "user.name")?;
        let git_email = utils::get_git_config(config, "user.email")?;

        if let (Some(git_user_name), Some(git_email)) = (git_user_name, git_email) {
            Some(format!("{} <{}>", git_user_name, git_email))
        } else {
            None
        }
    };

    let license = License::default();

    Ok(InitData {
        name,
        version,
        description,
        main,
        author,
        license,
        private: None,
    })
}

fn manual_initialization(default_name: String, config: &VoltConfig) -> Result<InitData> {
    // Get "name"
    let input = Input {
        message: "name".into(),
        default: Some(default_name.into()),
        allow_empty: false,
    };

    let re_name = Regex::new("^(?:@[a-z0-9-*~][a-z0-9-*._~]*/)?[a-z0-9-~][a-z0-9-._~]*$")
        .expect("Valid regex");
    let mut name;
    loop {
        name = input.run().into_diagnostic()?;

        if re_name.is_match(&name) {
            break;
        }

        println!("{}", "Name cannot contain special characters".red());
    }

    // Get "version"
    let input = Input {
        message: "version".into(),
        default: Some("1.0.0".into()),
        allow_empty: false,
    };

    let version = input.run().into_diagnostic()?;

    // Get "description"
    let input = Input {
        message: "description".into(),
        default: None,
        allow_empty: true,
    };

    let description = input.run().into_diagnostic()?;

    // Get "main"
    let input = Input {
        message: "main".into(),
        default: Some("index.js".into()),
        allow_empty: false,
    };

    let main = input.run().into_diagnostic()?;

    // Get "author"
    let git_user_name = utils::get_git_config(config, "user.name")?;

    let git_email = utils::get_git_config(config, "user.email")?;

    let input = Input {
        message: "author".into(),
        default: git_user_name
            .zip(git_email)
            .map(|(git_user_name, git_email)| format!("{} <{}>", git_user_name, git_email).into()),
        allow_empty: true,
    };

    let author = input.run().into_diagnostic()?;

    // Get "license"
    let select = Select {
        message: "License".into(),
        paged: true,
        selected: Some(1),
        items: License::OPTIONS.iter().map(|&l| l.into()).collect(),
    };

    select.run().into_diagnostic()?;

    let license = License::from_index(select.selected.unwrap()).unwrap();

    let input = Confirm {
        message: "private".into(),
        default: false,
    };

    let private = input.run().into_diagnostic()?;

    Ok(InitData {
        name,
        version,
        description: Some(description),
        main,
        author: Some(author),
        license,
        private: Some(private),
    })
}
