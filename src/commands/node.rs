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
    path::Path,
    process::Command,
    str,
};

use async_trait::async_trait;
use base64::decode;
use clap::{ArgMatches, Subcommand};
use futures::io;
use lzma_rs::xz_decompress;
//use lz4_flex;
use clap::Parser;
use miette::Result;
use node_semver::{Range, Version};
use serde::{Deserialize, Deserializer};
use tempfile::tempdir;
use tokio::fs;

use crate::cli::{VoltCommand, VoltConfig};

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
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum NodeCommand {
    Use(NodeUse),
    Install(NodeInstall),
    Remove(NodeRemove),
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
        if PLATFORM == Os::Windows {
            #[cfg(target_os = "windows")]
            use_windows(self.version).await;
        } else if PLATFORM == Os::Linux {
            let homedir = dirs::home_dir().unwrap();
            let node_path = format!("{}/.volt/Node/{}/node", homedir.display(), self.version);
            let path = Path::new(&node_path);

            if path.exists() {
                let link_dir = format!("{}/.local/bin", homedir.display());
                let link = format!("{}/{}", link_dir, "node.exe");
                //let symlink = std::os::unix::fs::symlink(node_path, link);
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

        let _node_versions: Vec<NodeVersion> = reqwest::get(format!("{}/index.json", mirror))
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        for v in self.versions {
            let mut download_url = format!("{}/", mirror);
            if v.parse::<Version>().is_ok() {
                if ARCH == Arch::X86 && (PLATFORM == Os::Macos || PLATFORM == Os::Linux) {
                    let major = v.split('.').next().unwrap().parse::<u8>().unwrap();

                    if major >= 10 {
                        println!("32 bit versions are not available for macos and linux after version 10.0.0!");
                        return Ok(());
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

                download_url = if cfg!(target_os = "windows") {
                    format!("{}/win-{}/node.exe", download_url, ARCH)
                } else {
                    format!("{}/node-v{}-{}-{}.tar.xz", download_url, v, PLATFORM, ARCH)
                };

                tracing::debug!("Got final URL '{}'", download_url);
            } else if v.parse::<Range>().is_ok() {
                //
                // TODO: Handle ranges with special chars like ^10.3
                //

                if ARCH == Arch::X86 && (PLATFORM == Os::Macos || PLATFORM == Os::Linux) {
                    let major = v.split('.').next().unwrap();
                    if major.parse::<u8>().unwrap() >= 10 {
                        println!("32 bit versions are not available for macos and linux after version 10.0.0!");
                        return Ok(());
                    }
                }
                todo!("Need to handle ranges");
            } else {
                println!("Unable to download {} -- not a valid version!", v);
                continue;
            }

            let node_path = {
                let datadir = dirs::data_dir().unwrap().join("volt").join("node");
                if !datadir.exists() {
                    fs::create_dir_all(&datadir).await.unwrap();
                }
                datadir
            };
            // Get the name of the directory the tarball unpacks to
            let unpack_loc = if cfg!(target_os = "windows") {
                &v // Windows locations are just saved in a folder named after the version number
                   // e.g. C:\Users\Alice\AppData\Roaming\volt\node\12.2.0
            } else {
                // The unix folders are just created by the tarball,
                // which is the basename of the file
                download_url
                    .split('/')
                    .last()
                    .unwrap()
                    .strip_suffix(".tar.xz")
                    .unwrap()
            };

            let node_path = node_path.join(unpack_loc);
            if node_path.exists() {
                println!("Node.js v{} is already installed, nothing to do!", v);
                continue;
            }

            tracing::debug!("Installing to: {:?}", node_path);

            // The name of the file we're downloading from the mirror
            let fname = download_url.split('/').last().unwrap().to_string();

            println!("Installing version {} from {} ", v, download_url);
            println!("file to download: '{}'", fname);

            let response = reqwest::get(&download_url).await.unwrap();

            let content = response.bytes().await.unwrap();

            if cfg!(target_os = "windows") {
                fs::create_dir_all(&node_path).await.unwrap();
                let mut dest = File::create(node_path.join(&fname)).unwrap();
                dest.write_all(&content).unwrap();
            } else {
                // TODO: Need to use https://github.com/fpgaminer/rust-lzma on linux
                // (if liblzma is widely available on distros) since it's way faster than
                // lzma_rs, but depends on the native lzma library being installed.
                // Need to check if `brew install xzip` will install it on osx, I think so.

                // File to write the download to
                let mut dest = File::create(dir.path().join(&fname)).unwrap();

                // File to write the decompressed tarball to
                let mut tarball =
                    File::create(dir.path().join(&fname.strip_suffix(".xz").unwrap())).unwrap();

                dest.write_all(&content).unwrap();

                xz_decompress(
                    &mut BufReader::new(File::open(dir.path().join(&fname)).unwrap()),
                    &mut tarball,
                )
                .unwrap();

                // Make sure the first file handle is closed
                drop(tarball);

                // Have to reopen it for reading, File::create() opens for write only
                let tarball =
                    File::open(dir.path().join(&fname.strip_suffix(".xz").unwrap())).unwrap();

                // Unpack the tarball to the right spot
                let mut ar = tar::Archive::new(tarball);
                ar.unpack(node_path).unwrap();
            }

            println!("\n---\n");
        }

        Ok(())
    }
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
        for version in self.versions {
            let node_path = dirs::data_dir()
                .unwrap()
                .join("volt")
                .join("node")
                .join(&version);

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
        }

        Ok(())
    }
}

#[cfg(target_os = "windows")]
async fn use_windows(version: String) {
    let homedir = dirs::home_dir().unwrap();
    let node_path = format!(
        "{}\\AppData\\Local\\Volt\\Node\\{}\\node.exe",
        homedir.display(),
        version
    );
    let path = Path::new(&node_path);

    if path.exists() {
        println!("Using version {}", version);
        let link_dir = format!("{}\\AppData\\Local\\Volt\\bin", homedir.display());
        fs::create_dir_all(&link_dir).await.unwrap();
        let link_file = format!("{}\\AppData\\Local\\Volt\\bin\\node.exe", homedir.display());
        let link_file = Path::new(&link_file);
        if link_file.exists() {
            fs::remove_file(link_file).await.unwrap();
        }
        let link = format!("{}\\{}", link_dir, "node.exe");
        println!("{}\n{}", node_path, link);

        let symlink = std::os::windows::fs::symlink_file(node_path, link);

        match symlink {
            Ok(_) => {}
            Err(_) => {
                println!("Error: \"volt node use\" must be run as an administrator on Windows!")
            }
        }

        let path = env::var("PATH").unwrap();
        //println!("{}", path);
        if !path.contains(&link_dir) {
            //env_perm::append("PATH", &link_dir);
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
