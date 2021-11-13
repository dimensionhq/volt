/*
 *    Copyright 2021 Volt Contributors
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

//! Manage local node versions

use clap::ArgMatches;
use miette::Result;
use node_semver::{Range, Version};
use serde::{Deserialize, Deserializer};
use tempfile::tempdir;
use tokio::fs;

use std::{fmt::Display, fs::File, io::Write, path::Path, str};

//use async_trait::async_trait;
//use colored::Colorize;

const PLATFORM: Os = if cfg!(target_os = "windows") {
    Os::Windows
} else if cfg!(target_os = "macos") {
    Os::Macos
} else if cfg!(target_os = "linux") {
    Os::Linux
} else {
    Os::Unknown
};

const ARCH: Arch = if cfg!(target_arch = "X86") {
    Arch::X86
} else if cfg!(target_arch = "x86_64") {
    Arch::X64
} else {
    Arch::Unknown
};

#[derive(Deserialize)]
#[serde(untagged)]
enum Lts {
    No(bool),
    Yes(String),
}

impl Into<Option<String>> for Lts {
    fn into(self) -> Option<String> {
        match self {
            Self::No(_) => None,
            Self::Yes(x) => Some(x),
        }
    }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Lts::deserialize(deserializer)?.into())
}

#[derive(Deserialize, Debug)]
pub struct NodeVersion {
    pub version: Version,
    #[serde(deserialize_with = "deserialize")]
    pub lts: Option<String>,
    pub files: Vec<String>,
}

#[derive(Debug, PartialEq)]
enum Os {
    Windows,
    Macos,
    Linux,
    Unknown,
}
impl Display for Os {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match &self {
            Os::Windows => "win",
            Os::Macos => "darwin",
            Os::Linux => "linux",
            _ => unreachable!(),
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, PartialEq)]
enum Arch {
    X86,
    X64,
    Unknown,
}

impl Display for Arch {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match *self {
            Arch::X86 => "x86",
            Arch::X64 => "x64",
            _ => unreachable!(),
        };
        write!(f, "{}", s)
    }
}

/// Struct implementation for the `Node` command.
#[derive(Clone)]
pub struct Node {}

impl Node {
    pub async fn download(args: &ArgMatches) -> Result<()> {
        match args.subcommand() {
            Some(("use", version)) => {
                println!("Using version {}", version.value_of("version").unwrap());
            }
            Some(("install", versions)) => {
                let v: Vec<&str> = versions.values_of("versions").unwrap().collect();
                download_node_version(v).await;
            }
            Some(("remove", versions)) => {
                let v: Vec<&str> = versions.values_of("versions").unwrap().collect();
                println!("Removing version {:?}", v);
            }
            _ => {}
        }
        Ok(())
    }
}

// 32bit macos/linux systems cannot download a version of node >= 10.0.0
// They stopped making 32bit builds after that version
// https://nodejs.org/dist/
// TODO: Handle errors with file already existing and handle file creation/deletion errors
// TODO: Only make a tempdir if we have versions to download, i.e. verify all versions before
//       creating the directory
async fn download_node_version(versions: Vec<&str>) {
    tracing::debug!("On platform '{}' and arch '{}'", PLATFORM, ARCH);
    let dir: tempfile::TempDir = tempdir().unwrap();
    tracing::debug!("Temp dir is {:?}", dir);

    let mirror = "https://nodejs.org/dist";

    let _node_versions: Vec<NodeVersion> = reqwest::get(format!("{}/index.json", mirror))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    for v in versions {
        let mut download_url = format!("{}/", mirror);
        if v.parse::<Version>().is_ok() {
            if ARCH == Arch::X86 && (PLATFORM == Os::Macos || PLATFORM == Os::Linux) {
                let major = v.split('.').next().unwrap().parse::<u8>().unwrap();

                if major >= 10 {
                    println!("32 bit versions are not available for macos and linux after version 10.0.0!");
                    return;
                }
            }

            let mut found = false;
            for n in &_node_versions {
                if v == n.version.to_string() {
                    download_url = format!("{}v{}", download_url, n.version);
                    found = true;
                    tracing::debug!("found version '{}' with URL '{}'", v, download_url);
                }
            }

            if !found {
                println!("Unable to find version {}!", v);
                continue;
            }

            if PLATFORM == Os::Windows {
                download_url = format!("{}/win-{}/node.exe", download_url, ARCH);
            } else {
                download_url = format!("{}/node-v{}-{}-{}.tar.gz", download_url, v, PLATFORM, ARCH);
            }

            tracing::debug!("Got final URL '{}'", download_url);
        } else if v.parse::<Range>().is_ok() {
            //
            // TODO: Handle ranges with special chars like ^10.3
            //

            if ARCH == Arch::X86 && (PLATFORM == Os::Macos || PLATFORM == Os::Linux) {
                let major = v.split('.').next().unwrap();
                if major.parse::<u8>().unwrap() >= 10 {
                    println!("32 bit versions are not available for macos and linux after version 10.0.0!");
                    return;
                }
            }
            todo!("Need to handle ranges");
        } else {
            println!("Unable to download {} -- not a valid version!", v);
            continue;
        }

        let node_path = {
            if PLATFORM == Os::Windows {
                let homedir = dirs::home_dir().unwrap();
                let node_path = format!("{}\\AppData\\Local\\Volt\\Node\\{}", homedir.display(), v);
                println!("Will install under: {}", node_path);
                fs::create_dir_all(&node_path).await.unwrap();

                format!("{}\\node.exe", &node_path)
            } else {
                println!("OS not supported.");
                continue;
            }
            /*
            else if (PLATFORM == Os::Linux) {
            }
            else if (PLATFORM == Os::Macos) {
            }
            */
        };

        if Path::new(&node_path).exists() {
            println!("Node.js v{} is already installed!", v);
            continue;
        }

        println!("Installing version {}", v);
        let response = reqwest::get(&download_url).await.unwrap();

        let mut dest = {
            let fname = response
                .url()
                .path_segments()
                .and_then(|segments| segments.last())
                .and_then(|name| if name.is_empty() { None } else { Some(name) })
                .unwrap();

            println!("file to download: '{}'", fname);
            let _fname = dir.path().join(fname.to_string());
            File::create(&node_path).unwrap()
        };

        let content = response.bytes().await.unwrap();

        dest.write_all(&content).unwrap();
        println!("\n---\n");
    }
}

/*#[async_trait]
  impl Command for Node {
/// Display a help menu for the `volt add` command.
fn help() -> String {
format!(
r#"volt {}

Manage NodeJS versions
Usage: {} {} {} {}
Options:

{} {} Output the version number.
{} {} Output verbose messages on internal operations.
{} {} Adds package as a dev dependency
{} {} Disable progress bar."#,
VERSION.bright_green().bold(),
"volt".bright_green().bold(),
"add".bright_purple(),
"[packages]".white(),
"[flags]".white(),
"--version".blue(),
"(-ver)".yellow(),
"--verbose".blue(),
"(-v)".yellow(),
"--dev".blue(),
"(-D)".yellow(),
"--no-progress".blue(),
"(-np)".yellow()
)
}

/// Execute the `volt node` command
///
/// Adds a package to dependencies for your project.
/// ## Arguments
/// * `app` - Instance of the command (`Arc<App>`)
/// ## Examples
/// ```rust
/// // Add react to your dependencies with logging level verbose
/// // .exec() is an async call so you need to await it
/// Add.exec(app).await;
/// ```
/// ## Returns
/// * `Result<()>`
async fn exec(app: Arc<App>) -> Result<()> {
println!("In Node Exec!");
let x = app.get_packages();
let x = x.unwrap();
for a in x {
println!("{:?}", a);
break;
}
Ok(())
}
}*/
