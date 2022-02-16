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

use std::{
    env,
    fmt::Display,
    fs::File,
    io::{BufReader, Write},
    path::{Path, PathBuf},
    process::Command,
    str,
    time::Duration,
};

use async_trait::async_trait;
use base64::decode;
use clap::Parser;
use clap::{ArgMatches, Subcommand};
use colored::Colorize;
use futures::io;
use indicatif::{ProgressBar, ProgressStyle};
use miette::Result;
use node_semver::{Range, Version};
use serde::{Deserialize, Deserializer};
use tempfile::tempdir;
use tokio::fs;

use crate::{
    cli::{VoltCommand, VoltConfig},
    core::utils::extensions::PathExtensions,
};

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

impl From<Lts> for Option<String> {
    fn from(val: Lts) -> Self {
        match val {
            Lts::No(_) => None,
            Lts::Yes(x) => Some(x),
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match *self {
            Arch::X86 => "x86",
            Arch::X64 => "x64",
            _ => unreachable!(),
        };
        write!(f, "{}", s)
    }
}

/// Manage node versions
#[derive(Debug, Parser)]
pub struct Node {
    #[clap(subcommand)]
    cmd: NodeCommand,
}

#[async_trait]
impl VoltCommand for Node {
    async fn exec(self, config: VoltConfig) -> Result<()> {
        match self.cmd {
            NodeCommand::Use(x) => x.exec(config).await,
            NodeCommand::Install(x) => x.exec(config).await,
            NodeCommand::Remove(x) => x.exec(config).await,
            NodeCommand::List(x) => x.exec(config).await,
        }
    }
}

#[derive(Debug, Parser)]
pub struct NodeList {}

fn sort_versions(a: &String, b: &String) -> std::cmp::Ordering {
    let a: Vec<i32> = a.split(".").map(|term| term.parse().unwrap()).collect();
    let b: Vec<i32> = b.split(".").map(|term| term.parse().unwrap()).collect();
    for ab in a.iter().zip(b.iter()) {
        let (ai, bi) = ab;
        let c = bi.cmp(ai);
        if c == std::cmp::Ordering::Equal {
            continue;
        } else {
            return c;
        }
    }
    return std::cmp::Ordering::Equal;
}

#[async_trait]
impl VoltCommand for NodeList {
    async fn exec(self, config: VoltConfig) -> Result<()> {
        let node_path = dirs::data_dir().unwrap().join("volt").join("node");
        let mut versions: Vec<String> = std::fs::read_dir(node_path)
            .unwrap()
            .map(|dir| match dir {
                Ok(_) => dir.unwrap().path().file_name_as_string().unwrap(),
                Err(_) => "ERROR".to_string(),
            })
            .filter(|x| x != "current")
            .collect();

        versions.sort_by(|a, b| sort_versions(a, b));
        println!(
            "Used version:\n\t{}",
            get_current_version().await.truecolor(250, 150, 100)
        );
        println!("Installed versions:");
        for version in versions {
            println!("\t{}", version.truecolor(250, 150, 100));
        }

        Ok(())
    }
}

#[derive(Debug, Subcommand)]
pub enum NodeCommand {
    Use(NodeUse),
    Install(NodeInstall),
    Remove(NodeRemove),
    List(NodeList),
}

/// Switch current node version
#[derive(Debug, Parser)]
pub struct NodeUse {
    /// Version to use
    version: String,
}

#[async_trait]
impl VoltCommand for NodeUse {
    async fn exec(self, config: VoltConfig) -> Result<()> {
        #[cfg(target_family = "windows")]
        use_windows(self.version).await;

        #[cfg(target_family = "unix")]
        {
            let node_path = get_node_path(&self.version);

            if node_path.exists() {
                let link_dir = dirs::home_dir().unwrap().join(".local").join("bin");

                let to_install = node_path.join("bin");
                let current = node_path.parent().unwrap().join("current");

                // TODO: Handle file deletion errors
                if current.exists() {
                    // Remove all the currently installed links
                    for f in std::fs::read_dir(&current).unwrap() {
                        let original = f.unwrap().file_name();
                        let installed = link_dir.join(&original);
                        if installed.exists() {
                            std::fs::remove_file(installed).unwrap();
                        }
                    }

                    // Remove the old link
                    std::fs::remove_file(&current).unwrap();

                    // Make a new one to the currently installed version
                    std::os::unix::fs::symlink(&to_install, current).unwrap();
                } else {
                    println!("Installing first version");
                    std::os::unix::fs::symlink(&to_install, current).unwrap();
                }

                for f in std::fs::read_dir(&to_install).unwrap() {
                    let original = f.unwrap().path();
                    let fname = original.file_name().unwrap();
                    let link = link_dir.join(fname);

                    println!("Linking to {:?} from {:?}", link, original);

                    // TODO: Do something with this error
                    let _ = std::fs::remove_file(&link);

                    // INFO: DOC: Need to run `rehash` in zsh for the changes to take effect
                    // maybe ship `vnm` as a shell function to run `volt node use ... && rehash` on
                    // zsh?
                    let _symlink = std::os::unix::fs::symlink(original, link).unwrap();
                }
            } else {
                println!("That version of node is not installed!\nTry \"volt node install {}\" to install that version.", self.version)
            }
        }

        Ok(())
    }
}

/// Install one or more versions of node
#[derive(Debug, Parser)]
pub struct NodeInstall {
    /// Versions to install
    versions: Vec<String>,
}

#[async_trait]
impl VoltCommand for NodeInstall {
    // 32bit macos/linux systems cannot download a version of node >= 10.0.0
    // They stopped making 32bit builds after that version
    // https://nodejs.org/dist/
    // TODO: Handle errors with file already existing and handle file creation/deletion errors
    // TODO: Only make a tempdir if we have versions to download, i.e. verify all versions before
    //       creating the directory
    async fn exec(self, _: VoltConfig) -> Result<()> {
        tracing::debug!("On platform '{}' and arch '{}'", PLATFORM, ARCH);
        let dir: tempfile::TempDir = tempdir().unwrap();
        tracing::debug!("Temp dir is {:?}", dir);

        let mirror = "https://nodejs.org/dist";

        let node_versions: Vec<NodeVersion> = reqwest::get(format!("{}/index.json", mirror))
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        for v in self.versions {
            let mut download_url = format!("{}/", mirror);
            let bar = ProgressBar::new_spinner()
                .with_style(ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}"));
            bar.set_message(format!(
                "{:10} {}",
                String::from("Installing"),
                v.truecolor(125, 125, 125)
            ));
            bar.enable_steady_tick(10);

            let version: Option<Version> = if let Ok(ver) = v.parse() {
                if cfg!(all(unix, target_arch = "X86")) && ver >= Version::parse("10.0.0").unwrap()
                {
                    println!("32 bit versions are not available for MacOS and Linux after version 10.0.0!");
                    continue;
                }

                // TODO: Maybe suggest the closest available version if not found?

                let mut found = false;
                for n in &node_versions {
                    if v == n.version.to_string() {
                        tracing::debug!("found version '{}' with URL '{}'", v, download_url);
                        found = true;
                        break;
                    }
                }

                if found {
                    Some(ver)
                } else {
                    None
                }
            } else if let Ok(ver) = v.parse::<Range>() {
                //volt install ^12
                let max_ver = node_versions
                    .iter()
                    .filter(|x| x.version.satisfies(&ver))
                    .map(|v| v.version.clone())
                    .max();

                if cfg!(all(unix, target_arch = "X86"))
                    && Range::parse(">=10").unwrap().allows_any(&ver)
                {
                    println!("32 bit versions are not available for macos and linux after version 10.0.0!");
                    continue;
                }

                max_ver
            } else {
                bar.finish_with_message(format!(
                    "{:10} {} ✗",
                    String::from("Invalid"),
                    v.to_string().truecolor(255, 000, 000)
                ));
                continue;
            };

            if version.is_none() {
                bar.finish_with_message(format!(
                    "{:10} {} ✗",
                    String::from("Not found"),
                    v.to_string().truecolor(255, 000, 000)
                ));
                continue;
            }

            let version = version.unwrap();

            download_url = format!("{}v{}/", download_url, version);

            download_url = if cfg!(target_os = "windows") {
                format!("{}/win-{}/node.exe", download_url, ARCH)
            } else {
                format!(
                    "{}node-v{}-{}-{}.tar.xz",
                    download_url, version, PLATFORM, ARCH
                )
            };

            //println!("\n------------\n{}\n------------\n", download_url);

            //println!("Got final URL '{}'", download_url);

            let node_path = {
                let datadir = dirs::data_dir().unwrap().join("volt").join("node");
                if !datadir.exists() {
                    std::fs::create_dir_all(&datadir).unwrap();
                }
                datadir
            };

            if node_path.join(version.to_string()).exists() {
                //println!("Node.js v{} is already installed, nothing to do!", version);
                bar.finish_with_message(format!(
                    "{:10} {} ✓",
                    String::from("Present"),
                    version.to_string().truecolor(000, 255, 000)
                ));
                continue;
            }

            tracing::debug!("Installing to: {:?}", node_path);

            // The name of the file we're downloading from the mirror
            let fname = download_url.split('/').last().unwrap().to_string();

            //println!("Installing version {} from {} ", version, download_url);
            //println!("file to download: '{}'", fname);

            let response = reqwest::get(&download_url).await.unwrap();

            let content = response.bytes().await.unwrap();

            #[cfg(target_family = "windows")]
            {
                let node_path = node_path.join(&version.to_string());
                //println!("Installing node.exe");
                std::fs::create_dir_all(&node_path).unwrap();
                let mut dest = File::create(node_path.join(&fname)).unwrap();
                dest.write_all(&content).unwrap();
            }

            #[cfg(target_family = "unix")]
            {
                // Path to write the decompressed tarball to
                let tarpath = &dir.path().join(&fname.strip_suffix(".xz").unwrap());

                //println!("Unzipping...");
                //println!("Tar path: {:?}", tarpath);
                //println!("HELLO WORLD");

                // Decompress the tarball
                let mut tarball = File::create(tarpath).unwrap();
                tarball
                    .write_all(&lzma::decompress(&content).unwrap())
                    .unwrap();

                // Make sure the first file handle is closed
                drop(tarball);

                // Have to reopen it for reading, File::create() opens for write only
                let tarball = File::open(&tarpath).unwrap();

                //println!("Unpacking...");

                // Unpack the tarball
                let mut w = tar::Archive::new(tarball);
                w.unpack(&node_path).unwrap();

                // TODO: Find a less disgusting way to do this?
                // Grab the name of the folder the tarball will extract to
                let p = tarpath
                    .file_name()
                    .unwrap()
                    .to_str()
                    .to_owned()
                    .unwrap()
                    .strip_suffix(".tar")
                    .unwrap();

                let from = node_path.join(&p);
                let to = node_path.join(&version.to_string());

                // Rename the folder from the default set by the tarball
                // to just the version number
                std::fs::rename(from, to);
            }
            bar.finish_with_message(format!(
                "{:10} {} ✓",
                String::from("Installed"),
                version.to_string().truecolor(000, 255, 000)
            ));
        }
        Ok(())
    }
}

fn get_node_path(version: &str) -> PathBuf {
    dirs::data_dir()
        .unwrap()
        .join("volt")
        .join("node")
        .join(&version)
}

/// Uninstall a specified version of node
#[derive(Debug, Parser)]
pub struct NodeRemove {
    /// Versions to remove
    versions: Vec<String>,
}

#[async_trait]
impl VoltCommand for NodeRemove {
    async fn exec(self, config: VoltConfig) -> Result<()> {
        let used_version = get_current_version().await;

        for version in self.versions {
            let node_path = get_node_path(&version);

            println!("{}", node_path.display());

            if node_path.exists() {
                fs::remove_dir_all(&node_path).await.unwrap();
                println!("Removed version {}", version);
            } else {
                println!(
                    "Failed to remove NodeJS version {}.\nThat version was not installed.",
                    version
                );
            }

            if used_version == version && PLATFORM == Os::Windows {
                let used_file = dirs::data_dir()
                    .unwrap()
                    .join("volt")
                    .join("bin")
                    .join("node.exe");
                std::fs::remove_file(used_file);
            }
        }

        Ok(())
    }
}

async fn get_current_version() -> String {
    let command = format!("node -v");
    let output = {
        if PLATFORM == Os::Windows {
            Command::new("Powershell")
                .args(&["-Command", &command])
                .output()
        } else {
            Command::new("sh").args(&["-c", &command]).output()
        }
    };

    let mut v = match output {
        Ok(_) => String::from_utf8(output.unwrap().stdout).unwrap(),
        Err(_) => String::from("vHidden (check file permissions)"),
    };

    if v == "" {
        v = String::from("vNone");
    }

    //trim trailing \r?\n (\r is windows only so this is an if statement)
    if v.ends_with('\n') {
        v.pop();
        if v.ends_with('\r') {
            v.pop();
        }
    }

    //trim leading 'v'
    return v[1..v.len()].to_string();
}

#[cfg(target_os = "windows")]
async fn use_windows(version: String) {
    let node_path = get_node_path(&version).join("node.exe");
    let path = Path::new(&node_path);

    if path.exists() {
        println!("Using version {}", version);

        let link_dir = dirs::data_dir()
            .unwrap()
            .join("volt")
            .join("bin")
            .into_os_string()
            .into_string()
            .unwrap();

        let link_file = dirs::data_dir()
            .unwrap()
            .join("volt")
            .join("bin")
            .join("node.exe");
        let link_file = Path::new(&link_file);

        if link_file.exists() {
            fs::remove_file(link_file).await.unwrap();
        }

        let newfile = std::fs::copy(node_path, link_file);

        match newfile {
            Ok(_) => {}
            Err(_) => {
                println!("Sorry, something went wrong.");
                return;
            }
        }

        let path = env::var("PATH").unwrap();
        if !path.contains(&link_dir) {
            let command = format!("[Environment]::SetEnvironmentVariable('Path', [Environment]::GetEnvironmentVariable('Path', 'User') + '{}', 'User')", &link_dir);
            Command::new("Powershell")
                .args(&["-Command", &command])
                .output()
                .unwrap();
            println!("PATH environment variable updated.\nYou will need to restart your terminal for changes to apply.");
        }
    } else {
        println!("That version of node is not installed!\nTry \"volt node install {}\" to install that version.", version);
    }
}
