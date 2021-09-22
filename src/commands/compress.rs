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

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread::{sleep, sleep_ms};
use std::time::{Duration, Instant};

use crate::App;
use crate::{core::VERSION, Command};
use async_trait::async_trait;
use colored::Colorize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use lazy_static::lazy_static;
use miette::{IntoDiagnostic, Result};
use regex::Regex;

lazy_static! {
    static ref REGEXES: Vec<Regex> = {
        vec![
            r"(?:/|\\)readme.*",
            r"(?:/|\\).npmignore",
            r"(?:/|\\)history.md",
            r"(?:/|\\)history.markdown",
            r"(?:/|\\).*.gitattributes",
            r"(?:/|\\).*.gitmodules",
            r"(?:/|\\).*.travis.yml",
            r"(?:/|\\)binding.gyp",
            r"(?:/|\\)contributing.*",
            r"(?:/|\\)component.json",
            r"(?:/|\\)composer.json",
            r"(?:/|\\)makefile.*",
            r"(?:/|\\)gemfile.*",
            r"(?:/|\\)rakefile.*",
            r"(?:/|\\).coveralls.yml",
            r"(?:/|\\)example.*",
            r"(?:/|\\)changelog.*",
            r"(?:/|\\)changes.*",
            r"(?:/|\\).jshintrc",
            r"(?:/|\\)bower.json",
            r"(?:/|\\)appveyor.yml",
            r"(?:/|\\).*.log",
            r"(?:/|\\).*.tlog",
            r"(?:/|\\).*.patch",
            r"(?:/|\\).*.sln",
            r"(?:/|\\).*.pdb",
            r"(?:/|\\).*.vcxproj",
            r"(?:/|\\).*.gitignore",
            r"(?:/|\\).*.sauce-labs",
            r"(?:/|\\).*.vimrc",
            r"(?:/|\\).*.idea",
            r"(?:/|\\)examples.*",
            r"(?:/|\\)samples.*",
            r"(?:/|\\)test.*",
            r"(?:/|\\)tests.*",
            "draft-00",
            "draft-01",
            "draft-02",
            "draft-03",
            "draft-04",
            r"(?:/|\\).*.eslintrc",
            r"(?:/|\\).*.jamignore",
            r"(?:/|\\).*.jscsrc",
            r"(?:/|\\).*.todo",
            r"(?:/|\\).*.md",
            r"(?:/|\\).*.js.map",
            r"(?:/|\\)contributors.*",
            r"(?:/|\\).orig",
            r"(?:/|\\).rej",
            r"(?:/|\\).zuul.yml",
            r"(?:/|\\).editorconfig",
            r"(?:/|\\).npmrc",
            r"(?:/|\\).jshintignore",
            r"(?:/|\\).eslintignore",
            r"(?:/|\\).lint",
            r"(?:/|\\).lintignore",
            "cakefile",
            r"(?:/|\\).istanbul.yml",
            "authors",
            "hyper-schema",
            "mocha.opts",
            r"(?:/|\\).*.gradle",
            r"(?:/|\\).tern-port",
            r"(?:/|\\).gitkeep",
            r"(?:/|\\).dntrc",
            r"(?:/|\\).watchr",
            r"(?:/|\\).jsbeautifyrc",
            "cname",
            "screenshots",
            r"(?:/|\\).dir-locals.el",
            "jsl.conf",
            "jsstyle",
            "benchmark",
            "dockerfile",
            r"(?:/|\\).nuspec",
            r"(?:/|\\).csproj",
            "thumbs.db",
            r"(?:/|\\).ds_store",
            "desktop.ini",
            "npm-debug.log",
            "wercker.yml",
            r"(?:/|\\).flowconfig",
        ]
        .into_iter()
        .map(|v| Regex::new(v).unwrap())
        .collect()
    };
}

pub struct Compress {}

pub fn minify(path: PathBuf) -> Result<()> {
    let start = Instant::now();

    let file = File::open(&path).into_diagnostic()?;
    let mut reader = minifier::json::minify_from_read(&file);

    let mut out_file = tempfile::NamedTempFile::new().unwrap();

    std::io::copy(&mut reader, &mut out_file).unwrap();
    out_file.persist(&path).unwrap();
    // println!("{}", out_file.path().display());
    // std::io::copy(&mut reader, &mut out_file);
    // std::fs::rename(out_file.path(), path).unwrap();

    // file.read_to_string(&mut contents).into_diagnostic()?;

    // let minified = minifier::json::minify(&contents);

    // file.set_len(0).into_diagnostic()?;
    // file.rewind().into_diagnostic()?;

    // file.write_all(minified.as_bytes()).into_diagnostic()?;
    println!("{}", start.elapsed().as_secs_f32());

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

        for entry in jwalk::WalkDir::new("node_modules") {
            let path = entry.unwrap().path();

            let path_str = path.to_str().unwrap().to_string().to_lowercase();
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

        let minify_bar = ProgressBar::new(minify_files.len() as u64);
        minify_bar.set_style(
            ProgressStyle::default_bar()
                .template("Minifying Files - [{bar:.green/magenta}] {pos} / {len} {per_sec}")
                .progress_chars("=>-"),
        );

        for file in minify_files {
            minify(file).unwrap();
            minify_bar.inc(1);
        }

        minify_bar.finish();

        Ok(())
    }
}
