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

//! Clean `./node_modules` and reduce its size.

use crate::cli::{VoltCommand, VoltConfig};

use async_trait::async_trait;
use clap::Parser;
use colored::Colorize;
use futures::{stream::FuturesUnordered, StreamExt};
use indicatif::{HumanBytes, ProgressBar, ProgressStyle};
use miette::{IntoDiagnostic, Result};
use regex::Regex;
use std::{
    fs,
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::task::spawn_blocking;

/// Optimizes your `./node_modules` by removing redundant files and folders
#[derive(Debug, Parser)]
pub struct Clean {
    /// Remove license file from the packages
    #[clap(short, long)]
    remove_licenses: bool,
}

#[async_trait]
impl VoltCommand for Clean {
    /// Execute the `volt clean` command
    ///
    /// Clean node_modules and removes redundant files.
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
    async fn exec(self, _config: VoltConfig) -> Result<()> {
        let regexes = get_regexes(self.remove_licenses);

        let mut matches: Vec<PathBuf> = vec![];
        let mut minify_files: Vec<PathBuf> = vec![];
        let mut node_modules_contents: Vec<PathBuf> = vec![];

        let mut workers = FuturesUnordered::new();

        let mut initial_file_size: u64 = 0;
        let mut final_file_size: u64 = 0;

        for entry in jwalk::WalkDir::new("node_modules") {
            let path = entry.into_diagnostic()?.path();
            node_modules_contents.push(path.clone());
        }

        let regexes = Arc::new(regexes);

        for chunks in node_modules_contents.chunks(150) {
            let chunk = chunks.to_vec();

            let regexes = regexes.clone();

            workers.push(tokio::task::spawn_blocking(move || {
                let mut regex_matches = vec![];
                let mut minify_matches = vec![];
                let mut initial_size: u64 = 0;

                'path: for path in chunk {
                    initial_size += path.metadata().unwrap().len();

                    let path_str = path.to_str().unwrap().replace('\\', "/").to_lowercase();

                    for regex in regexes.iter() {
                        if regex.is_match(&path_str) {
                            regex_matches.push(path);
                            continue 'path;
                        }
                    }

                    if let Some(extension) = path.extension() {
                        if extension.to_str().unwrap() == "json" {
                            minify_matches.push(path);
                        }
                    }
                }

                (minify_matches, regex_matches, initial_size)
            }));
        }

        while let Some((minify_matches, regex_matches, initial_size)) =
            workers.next().await.transpose().into_diagnostic()?
        {
            minify_files.extend(minify_matches);
            matches.extend(regex_matches);
            initial_file_size += initial_size;
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
            let chunk_instance = chunk.to_vec();
            workers.push(spawn_blocking(move || -> Result<()> {
                for file in chunk_instance {
                    minify(&file)?;
                }
                Ok(())
            }));
        }

        while workers
            .next()
            .await
            .transpose()
            .into_diagnostic()?
            .transpose()?
            .is_some()
        {
            minify_bar.inc(20);
        }

        minify_bar.finish();

        let mut workers = FuturesUnordered::new();

        for chunk in matches.chunks(90) {
            let chunk = chunk.to_vec();
            let matches_bar = matches_bar.clone();

            workers.push(tokio::task::spawn_blocking(move || -> Result<()> {
                for entry in chunk {
                    if entry.is_file() {
                        fs::remove_file(&entry).into_diagnostic()?;
                    } else if entry.is_dir() {
                        fs::remove_dir_all(&entry).into_diagnostic()?;
                    }
                    matches_bar.inc(1);
                }
                Ok(())
            }));
        }

        while workers
            .next()
            .await
            .transpose()
            .into_diagnostic()?
            .transpose()?
            .is_some()
        {}

        matches_bar.finish();

        let mut workers = FuturesUnordered::new();

        for chunk in node_modules_contents.chunks(200) {
            let chunk = chunk.to_vec();

            workers.push(tokio::task::spawn_blocking(move || -> Result<()> {
                for entry in chunk {
                    if entry.is_dir() && entry.read_dir().into_diagnostic()?.next().is_none() {
                        fs::remove_dir(entry).into_diagnostic()?;
                    }
                }
                Ok(())
            }));
        }

        while workers
            .next()
            .await
            .transpose()
            .into_diagnostic()?
            .transpose()?
            .is_some()
        {}

        node_modules_contents.clear();

        for entry in jwalk::WalkDir::new("node_modules") {
            let path = entry.unwrap().path();
            node_modules_contents.push(path.clone());
        }

        let mut workers = FuturesUnordered::new();

        for chunks in node_modules_contents.chunks(150) {
            let chunk = chunks.to_vec();

            workers.push(tokio::task::spawn_blocking(move || -> Result<u64> {
                let mut final_size: u64 = 0;

                for entry in chunk {
                    if entry.is_dir() {
                        final_size += entry.metadata().into_diagnostic()?.len();
                    } else {
                        final_size += fs_extra::dir::get_size(entry).into_diagnostic()?;
                    }
                }

                Ok(final_size)
            }));
        }

        while let Some(value) = workers
            .next()
            .await
            .transpose()
            .into_diagnostic()?
            .transpose()?
        {
            final_file_size += value;
        }

        let removed_size = initial_file_size - final_file_size;

        println!(
            "{} {} {} ( {} Saved )",
            HumanBytes(initial_file_size),
            "->".bright_magenta().bold(),
            HumanBytes(final_file_size),
            HumanBytes(removed_size).to_string().bright_green(),
        );

        Ok(())
    }
}

// minify a JSON file
fn minify(path: &Path) -> Result<()> {
    let mut contents = String::new();

    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .into_diagnostic()?;

    file.read_to_string(&mut contents).into_diagnostic()?;

    let minified = minifier::json::minify(&contents);

    file.set_len(0).into_diagnostic()?;
    file.seek(SeekFrom::Start(0)).into_diagnostic()?;

    file.write_all(minified.as_bytes()).into_diagnostic()?;

    Ok(())
}

fn get_regexes(remove_licenses: bool) -> Box<[Regex]> {
    let mut regexes: Vec<Regex> = {
        vec![
            r"^.*/readme(?:.md|.txt|.markdown)?$",
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
        .map(|v| Regex::new(v).expect("Valid regex"))
        .collect()
    };

    // Append the LICENSE regexes if the flag is specified
    if remove_licenses {
        regexes.push(Regex::new(r"^.*/license(?:.md|.txt|.markdown)?$").expect("Valid regex"));
    }

    regexes.into_boxed_slice()
}

//
// Benchmarks:

// > 0.021818367  (old code, using async and tokio workers)
// > 0.00535743
