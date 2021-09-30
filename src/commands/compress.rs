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

use crate::App;
use crate::{core::VERSION, Command};
use async_trait::async_trait;
use colored::Colorize;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use lazy_static::lazy_static;
use miette::{IntoDiagnostic, Result};
use regex::Regex;
use std::fs;
use std::io::SeekFrom;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

lazy_static! {
    static ref REGEXES: Vec<Regex> = {
        vec![
            r"^.*/readme(?:.md|.txt|.markdown)?$",
            r"^.*/.npmignore$",
            r"^.*/history(?:.md|.txt|.markdown)?$",
            r"^.*/.gitattributes$",
            r"^.*/.gitmodules$",
            r"^.*/.travis.yml$",
            r"^.*/.binding.gyp$",
            r"^.*/contributing(?:.md|.txt|.markdown)?$",
            r"^.*/composer.json$",
            r"^.*/makefile$",
            r"^.*/gemfile$",
            r"^.*/rakefile$",
            r"^.*/.coveralls.yml$",
            r"^.*/examples?/.*$",
            r"^.*/changelog(?:.md|.txt|.markdown)?$",
            r"^.*/changes(?:.md|.txt|.markdown)?$",
            r"^.*/.jshintrc$",
            r"^.*/bower.json$",
            r"^.*/appveyor.yml$",
            r"^.*/.*.log$",
            r"^.*/.*.tlog$",
            r"^.*/.*.patch$",
            r"^.*/.*.sln$",
            r"^.*/.*.pdb$",
            r"^.*/.*.vcxproj$",
            r"^.*/.*.gitignore$",
            r"^.*/.*.vimrc$",
            r"^.*/.*.idea$",
            r"^.*/samples?/.*$",
            r"^.*/tests?/.*$",
            r"^.*/.eslintrc$",
            r"^.*/.jamignore$",
            r"^.*/.jscsrc$",
            r"^.*/.*.todo$",
            r"^.*/.*.js.map$",
            r"^.*/contributors(?:.md|.txt|.markdown)?$",
            r"^.*/.*.orig$",
            r"^.*/.*.rej$",
            r"^.*/.zuul.yml$",
            r"^.*/.editorconfig$",
            r"^.*/.npmrc$",
            r"^.*/.jshintignore$",
            r"^.*/.eslintignore$",
            r"^.*/.*.lint$",
            r"^.*/.*.lintignore$",
            r"^.*/cakefile$",
            r"^.*/.istanbul.yml$",
            r"^.*/mocha.opts$",
            r"^.*/.*.gradle$",
            r"^.*/.*.tern-port$",
            r"^.*/.gitkeep$",
            r"^.*/.dntrc$",
            r"^.*/.*.watchr$",
            r"^.*/.jsbeautifyrc$",
            r"^.*/cname$",
            r"^.*/screenshots?/.*$",
            r"^.*/.dir-locals.el$",
            r"^.*/jsl.conf$",
            r"^.*/jsstyle$",
            r"^.*/benchmarks?/.*$",
            r"^.*/dockerfile$",
            r"^.*/.*.nuspec$",
            r"^.*/.*.csproj$",
            r"^.*/thumbs.db$",
            r"^.*/.ds_store$",
            r"^.*/desktop.ini$",
            r"^.*/npm-debug.log$",
            r"^.*/.wercker.yml$",
            r"^.*/.flowconfig$",
        ]
        .into_iter()
        .map(|v| Regex::new(v).unwrap())
        .collect()
    };
}

pub struct Compress {}

// minify a JSON file
pub async fn minify(path: &PathBuf) -> Result<()> {
    let mut contents = String::new();

    let mut file = tokio::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .await
        .into_diagnostic()?;

    file.read_to_string(&mut contents).await.into_diagnostic()?;

    let minified = minifier::json::minify(&contents);

    file.set_len(0).await.into_diagnostic()?;
    file.seek(SeekFrom::Start(0)).await.into_diagnostic()?;

    file.write_all(minified.as_bytes())
        .await
        .into_diagnostic()?;

    Ok(())
}

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
        let mut matches: Vec<PathBuf> = vec![];
        let mut minify_files: Vec<PathBuf> = vec![];
        let mut node_modules_contents: Vec<PathBuf> = vec![];

        for entry in jwalk::WalkDir::new("node_modules") {
            let path = entry.unwrap().path();
            node_modules_contents.push(path.clone());

            let path_str = path
                .to_str()
                .unwrap()
                .to_string()
                .replace(r"\", "/")
                .to_lowercase();
            let mut has_match = false;

            for regex in REGEXES.iter() {
                if regex.is_match(&path_str) {
                    matches.push(path.clone());
                    has_match = true;
                    break;
                };
            }

            if !has_match {
                if let Some(extension) = path.extension() {
                    match extension.to_str().unwrap() {
                        "json" => {
                            minify_files.push(path.clone());
                        }
                        _ => {}
                    }
                }
            }
        }

        let matches_bar = ProgressBar::new(matches.len() as u64);
        let minify_bar = ProgressBar::new(minify_files.len() as u64);

        minify_bar.set_style(
            ProgressStyle::default_bar()
                .template("Minifying Files - [{bar:.green/magenta}] {pos} / {len} {per_sec}")
                .progress_chars("=>-"),
        );

        matches_bar.set_style(
            ProgressStyle::default_bar()
                .template("Deleting Non-Essential Files & Folders - [{bar:.green/magenta}] {pos} / {len} {per_sec}")
                .progress_chars("=>-"),
        );

        let mut workers = FuturesUnordered::new();

        let minify_bar = minify_bar.clone();

        for chunk in minify_files.chunks(20) {
            workers.push(async move {
                for file in chunk {
                    minify(file).await.unwrap();
                }
            });
        }

        while workers.next().await.is_some() {
            minify_bar.inc(20);
        }

        let mut workers = FuturesUnordered::new();

        minify_bar.finish();

        for chunk in matches.chunks(90) {
            let chunk = chunk.to_vec();
            let matches_bar = matches_bar.clone();

            workers.push(tokio::task::spawn_blocking(move || {
                for entry in chunk {
                    if entry.is_file() {
                        match fs::remove_file(entry) {
                            Ok(_) => matches_bar.inc(1),
                            Err(_) => {}
                        };
                    } else if entry.is_dir() {
                        match fs::remove_dir_all(entry) {
                            Ok(_) => matches_bar.inc(1),
                            Err(_) => {}
                        };
                    }
                }
            }));
        }

        while workers.next().await.is_some() {}

        matches_bar.finish();

        let mut workers = FuturesUnordered::new();

        for chunk in node_modules_contents.chunks(200) {
            let chunk = chunk.to_vec();

            workers.push(tokio::task::spawn_blocking(move || {
                for entry in chunk {
                    if entry.is_dir() && entry.read_dir().unwrap().next().is_none() {
                        match fs::remove_dir(entry) {
                            Ok(_) => {}
                            Err(_) => {}
                        };
                    }
                }
            }))
        }

        while workers.next().await.is_some() {}

        Ok(())
    }
}
