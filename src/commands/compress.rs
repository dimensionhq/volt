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

//! Compress node_modules into node_modules.pack.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use crate::App;
use crate::{core::VERSION, Command};
use async_trait::async_trait;
use colored::Colorize;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use glob::MatchOptions;
use globset::{Glob, GlobBuilder, GlobMatcher, GlobSetBuilder};
use globwalk::{DirEntry, GlobWalker};
use miette::Result;
use regex::Regex;
pub struct Compress {}

#[async_trait]
impl Command for Compress {
    /// Display a help menu for the `volt compress` command.
    fn help() -> String {
        format!(
            r#"volt {}
    
Compress node_modules into node_modules.pack.
Usage: {} {} {} {}
Options: 
    
  {} {} Output verbose messages on internal operations.
  {} {} Disable progress bar."#,
            VERSION.bright_green().bold(),
            "volt".bright_green().bold(),
            "clone".bright_purple(),
            "[repository]".white(),
            "[flags]".white(),
            "--verbose".blue(),
            "(-v)".yellow(),
            "--no-progress".blue(),
            "(-np)".yellow()
        )
    }

    /// Execute the `volt compress` command
    ///
    /// Compress node_modules into node_modules.pack.
    /// ## Arguments
    /// * `app` - Instance of the command (`Arc<App>`)
    /// ## Examples
    /// ```
    /// //  Optimizes your node_modules by removing redundant files and folders
    /// // .exec() is an async call so you need to await it
    /// Add.exec(app).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(_app: Arc<App>) -> Result<()> {
        let start = Instant::now();

        let removables = vec![
            "readme.*",
            ".*.npmignore",
            "license",
            "license.md",
            "licence.md",
            "license.markdown",
            "licence.markdown",
            "license-mit",
            "history.md",
            "history.markdown",
            ".*.gitattributes",
            ".*.gitmodules",
            ".*.travis.yml",
            "binding.gyp",
            "contributing.*",
            "component.json",
            "composer.json",
            "makefile.*",
            "gemfile.*",
            "rakefile.*",
            ".*.coveralls.yml",
            "example.*",
            "changelog.*",
            "changes.*",
            ".*.jshintrc",
            "bower.json",
            "appveyor.yml",
            ".*.log",
            ".*.tlog",
            ".*.patch",
            ".*.sln",
            ".*.pdb",
            ".*.vcxproj",
            ".*.gitignore",
            ".*.sauce-labs",
            ".*.vimrc",
            ".*.idea",
            "examples.*",
            "samples.*",
            "test.*",
            "tests.*",
            "draft-00",
            "draft-01",
            "draft-02",
            "draft-03",
            "draft-04",
            ".*.eslintrc",
            ".*.jamignore",
            ".*.jscsrc",
            ".*.todo",
            ".*.md",
            ".*.js.map",
            ".*.js.map",
            "contributors.*",
            ".*.orig",
            ".*.rej",
            ".*.zuul.yml",
            ".*.editorconfig",
            ".*.npmrc",
            ".*.jshintignore",
            ".*.eslintignore",
            ".*.lint",
            ".*.lintignore",
            "cakefile",
            ".*.istanbul.yml",
            "authors",
            "hyper-schema",
            "mocha.opts",
            ".*.gradle",
            ".*.tern-port",
            ".*.gitkeep",
            ".*.dntrc",
            ".*.watchr",
            ".*.jsbeautifyrc",
            "cname",
            "screenshots",
            ".*.dir-locals.el",
            "jsl.conf",
            "jsstyle",
            "benchmark",
            "dockerfile",
            ".*.nuspec",
            ".*.csproj",
            "thumbs.db",
            ".*.ds_store",
            "desktop.ini",
            "npm-debug.log",
            "wercker.yml",
            ".*.flowconfig",
        ];

        for entry in jwalk::WalkDir::new("node_modules") {
            let path = entry
                .unwrap()
                .path()
                .to_str()
                .unwrap()
                .to_string()
                .replace(r"node_modules\", "")
                .to_lowercase();

            println!("{}", path);
        }

        Ok(())
    }
}
