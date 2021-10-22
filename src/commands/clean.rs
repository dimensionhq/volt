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

//! Clean ./node_modules and reduce its size.

use crate::{core::VERSION, App, Command};
use async_trait::async_trait;
use colored::Colorize;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use indicatif::{HumanBytes, ProgressBar, ProgressStyle};
use lazy_static::lazy_static;
use miette::{IntoDiagnostic, Result};
use regex::Regex;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

use std::{
    fs,
    io::SeekFrom,
    path::{Path, PathBuf},
    sync::Arc,
};

lazy_static! {
    static ref REGEXES: Vec<Regex> = {
        vec![
            r"^.*/readme(?:.md|.txt|.markdown)?$",
            r"^.*/readme.zh(?:.md|.txt|.markdown)?$",
            r"^.*/.npmignore$",
            r"^.*/yarn.lock$",
            r"^.*/npm-lock.json$",
            r"^.*/history(?:.md|.txt|.markdown)?$",
            r"^.*/security(?:.md|.txt|.markdown)?$",
            r"^.*/.gitattributes$",
            r"^.*/.gitmodules$",
            r"^.*/.prettierrc$",
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
            r"^.*/testing/.*$",
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
            r"^.*/.*.md$",
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

pub struct Clean {}

// minify a JSON file
pub async fn minify(path: &Path) -> Result<()> {
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
impl Command for Clean {
    /// Display a help menu for the `volt clean` command.
    fn help() -> String {
        format!(
            r#"volt {}
    
Clean ./node_modules and reduce its size.
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

    /// Execute the `volt clean` command
    ///
    /// Clean node_modules into node_modules.pack.
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

        let mut workers = FuturesUnordered::new();

        let mut initial_file_size: u64 = 0;
        let mut final_file_size: u64 = 0;

        for entry in jwalk::WalkDir::new("node_modules") {
            let path = entry.unwrap().path();
            node_modules_contents.push(path.clone());
        }

        for chunks in node_modules_contents.chunks(150) {
            let chunk = chunks.to_vec();

            workers.push(tokio::task::spawn_blocking(move || {
                let mut regex_matches = vec![];
                let mut minify_matches = vec![];
                let mut initial_size: u64 = 0;

                for path in chunk {
                    initial_size += path.metadata().unwrap().len();

                    let path_str = path
                        .to_str()
                        .unwrap()
                        .to_string()
                        .replace('\\', "/")
                        .to_lowercase();

                    let mut has_match = false;

                    for regex in REGEXES.iter() {
                        if path_str.contains("npmignore") {
                            println!("{}", path_str);
                        }

                        if regex.is_match(&path_str) {
                            regex_matches.push(path.clone());
                            has_match = true;
                            break;
                        };
                    }

                    if !has_match {
                        if let Some(extension) = path.extension() {
                            if extension.to_str().unwrap() == "json" {
                                minify_matches.push(path.clone());
                            }
                        }
                    }
                }

                (minify_matches, regex_matches, initial_size)
            }));
        }

        while let Some(Ok(value)) = workers.next().await {
            minify_files.extend(value.0);
            matches.extend(value.1);
            initial_file_size += value.2;
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

        for chunk in minify_files.chunks(20) {
            workers.push(async move {
                for file in chunk {
                    minify(file).await.unwrap_or_else(|v| {
                        println!("{}", v);
                    });
                }
            });
        }

        while workers.next().await.is_some() {
            minify_bar.inc(20);
        }

        minify_bar.finish();

        let mut workers = FuturesUnordered::new();

        for chunk in matches.chunks(90) {
            let chunk = chunk.to_vec();
            let matches_bar = matches_bar.clone();

            workers.push(tokio::task::spawn_blocking(move || {
                for entry in chunk {
                    if entry.is_file() && fs::remove_file(&entry).is_ok()
                        || entry.is_dir() && fs::remove_dir_all(&entry).is_ok()
                    {
                        matches_bar.inc(1)
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
                    if entry.is_dir()
                        && entry.read_dir().unwrap().next().is_none()
                        && fs::remove_dir(entry).is_ok()
                    {}
                }
            }))
        }

        while workers.next().await.is_some() {}

        node_modules_contents.clear();

        for entry in jwalk::WalkDir::new("node_modules") {
            let path = entry.unwrap().path();
            node_modules_contents.push(path.clone());
        }

        let mut workers = FuturesUnordered::new();

        for chunks in node_modules_contents.chunks(150) {
            let chunk = chunks.to_vec();

            workers.push(tokio::task::spawn_blocking(move || {
                let mut final_size: u64 = 0;

                for entry in chunk {
                    if entry.is_dir() {
                        final_size += entry.metadata().unwrap().len();
                    } else {
                        final_size += fs_extra::dir::get_size(entry).unwrap();
                    }
                }

                final_size
            }));
        }

        while let Some(Ok(value)) = workers.next().await {
            final_file_size += value;
        }

        let removed_size = initial_file_size - final_file_size;

        println!(
            "{} {} {} ( {} Saved )",
            HumanBytes(initial_file_size).to_string(),
            "->".bright_magenta().bold(),
            HumanBytes(final_file_size).to_string(),
            HumanBytes(removed_size).to_string().bright_green(),
        );

        Ok(())
    }
}
