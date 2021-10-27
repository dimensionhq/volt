/*
<<<<<<< HEAD
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

//! Manage local node versions

use crate::{
    core::{command::Command, VERSION},
    App,
};

use std::fmt::Display;
use std::fs::File;
use std::io::copy;
use std::io::prelude::*;
use std::sync::Arc;

use async_trait::async_trait;
use clap::ArgMatches;
use colored::Colorize;
use miette::Result;
use node_semver::{Range, Version};
use serde::{Deserialize, Deserializer};
=======
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

//! Manage local node versions
/*use crate::{
    core::{command::Command, VERSION},
    App,
};*/

use std::str;
use std::{fmt::Display, fs::File, io::Write};

use clap::ArgMatches;
use miette::Result;
use node_semver::{Range, Version};
use serde::{Deserialize, Deserializer};
use tempfile::tempdir;
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
>>>>>>> e56eb72395529f1dd21931ce99282f6812000e6a

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
<<<<<<< HEAD
=======

>>>>>>> e56eb72395529f1dd21931ce99282f6812000e6a
impl Display for Os {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            &Os::Windows => "win",
            &Os::Macos => "darwin",
            &Os::Linux => "linux",
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
        let s = match self {
            &Arch::X86 => "x86",
            &Arch::X64 => "x64",
            _ => unreachable!(),
        };
        write!(f, "{}", s)
    }
}

<<<<<<< HEAD
fn get_arch() -> Arch {
    // "x86", "x86_64", "mips", "powerpc", "powerpc64", "arm", or "aarch64".
    if cfg!(target_arch = "x86") {
        Arch::X86
    } else if cfg!(target_arch = "x86_64") {
        Arch::X64
    } else {
        Arch::Unknown
    }
}

fn get_platform() -> Os {
    if cfg!(target_os = "windows") {
        Os::Windows
    } else if cfg!(target_os = "macos") {
        Os::Macos
    } else if cfg!(target_os = "linux") {
        Os::Linux
    } else {
        Os::Unknown
    }
}

/// Struct implementation for the `Add` command.
=======
/// Struct implementation for the `Node` command.
>>>>>>> e56eb72395529f1dd21931ce99282f6812000e6a
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
<<<<<<< HEAD
                println!("Installing version {:?}", v);
=======
>>>>>>> e56eb72395529f1dd21931ce99282f6812000e6a
                download_node_version(v).await;
            }
            Some(("remove", versions)) => {
                let v: Vec<&str> = versions.values_of("versions").unwrap().collect();
                println!("Removing version {:?}", v);
            }
            _ => {}
        }
<<<<<<< HEAD

=======
>>>>>>> e56eb72395529f1dd21931ce99282f6812000e6a
        Ok(())
    }
}

// 32bit macos/linux systems cannot download a version of node >= 10.0.0
// They stopped making 32bit builds after that version
// https://nodejs.org/dist/
<<<<<<< HEAD
async fn download_node_version(versions: Vec<&str>) {
    let arch = get_arch();
    let os = get_platform();

    let _v10 = "10".parse::<Range>().unwrap();
=======
// TODO: Handle errors with file already existing and handle file creation/deletion errors
async fn download_node_version(versions: Vec<&str>) {
    // TODO: Only make a tempdir if we have versions to download, i.e. verify all versions before
    // creating the directory
    let dir: tempfile::TempDir = tempdir().unwrap();
    println!("Got tempdir: {}", dir.path().to_str().unwrap());
>>>>>>> e56eb72395529f1dd21931ce99282f6812000e6a

    let mirror = "https://nodejs.org/dist";

    let _node_versions: Vec<NodeVersion> = reqwest::get(format!("{}/index.json", mirror))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    for v in versions {
        let mut download_url = format!("{}/", mirror);
        if let Ok(_) = v.parse::<Version>() {
<<<<<<< HEAD
            if arch == Arch::X86 && (os == Os::Macos || os == Os::Linux) {
=======
            if ARCH == Arch::X86 && (PLATFORM == Os::Macos || PLATFORM == Os::Linux) {
>>>>>>> e56eb72395529f1dd21931ce99282f6812000e6a
                let major = v.split('.').next().unwrap().parse::<u8>().unwrap();

                if major >= 10 {
                    println!("32 bit versions are not available for macos and linux after version 10.0.0!");
                    return;
                }
            }

            let mut found = false;
            for n in &_node_versions {
                if v == n.version.to_string() {
                    // println!("Found matching version: {:?}", n);
                    download_url = format!("{}v{}", download_url, n.version);
                    found = true;
                }
            }

            if !found {
                println!("Unable to find version {}!", v);
                return;
            }

<<<<<<< HEAD
            if os == Os::Windows {
                download_url = format!("{}/win-{}/node.exe", download_url, arch);
            } else {
                download_url = format!("{}/node-v{}-{}-{}.tar.gz", download_url, v, os, arch);
            }
            dbg!(&download_url);
            /*
             * println!(
             *     "Installing version {:?} for architecture {} for OS {}",
             *     x, arch, os
             * );
             */
=======
            if PLATFORM == Os::Windows {
                download_url = format!("{}/win-{}/node.exe", download_url, ARCH);
            } else {
                download_url = format!("{}/node-v{}-{}-{}.tar.gz", download_url, v, PLATFORM, ARCH);
            }
>>>>>>> e56eb72395529f1dd21931ce99282f6812000e6a
        } else if let Ok(_) = v.parse::<Range>() {
            //
            // TODO: Handle ranges with special chars like ^10.3
            //

<<<<<<< HEAD
            if arch == Arch::X86 && os == Os::Macos || os == Os::Linux {
=======
            if ARCH == Arch::X86 && (PLATFORM == Os::Macos || PLATFORM == Os::Linux) {
>>>>>>> e56eb72395529f1dd21931ce99282f6812000e6a
                let major = v.split('.').next().unwrap();
                if major.parse::<u8>().unwrap() >= 10 {
                    println!("32 bit versions are not available for macos and linux after version 10.0.0!");
                    return;
                }
            }
<<<<<<< HEAD
        } else {
            println!("Unable to downlaod {} -- not a valid version!", v);
            continue;
        }

=======
            todo!("Need to handle ranges");
        } else {
            println!("Unable to download {} -- not a valid version!", v);
            continue;
        }

        println!("Installing version {}", v);
>>>>>>> e56eb72395529f1dd21931ce99282f6812000e6a
        let response = reqwest::get(&download_url).await.unwrap();

        let mut dest = {
            let fname = response
                .url()
                .path_segments()
                .and_then(|segments| segments.last())
                .and_then(|name| if name.is_empty() { None } else { Some(name) })
<<<<<<< HEAD
                .unwrap_or("tmp.bin");

            println!("file to download: '{}'", fname);
            let fname = format!("Downloads/{}", fname);
            println!("will be located under: '{:?}'", fname);
            File::create(fname).unwrap()
        };
        let content = response.bytes().await.unwrap();
        //copy(&mut content.bytes(), &mut dest).unwrap();
        dest.write_all(&content);
        println!("{}", download_url);
    }

    /*
     * let mut file = std::fs::File::create("file.tar.gz").unwrap();
     * let mut content = Cursor::new(response.await.unwrap().bytes().await.unwrap());
     * std::io::copy(&mut content, &mut file).unwrap();
     */

    /*
     *     let lts = node_versions.iter().filter(|x| x.lts.is_some());
     *
     *     for l in lts {
     *         println!("{:?}", l);
     *     }
     */

    // let response = reqwest::get("https://nodejs.org/dist/node-v0.1.25.tar.gz");

    /*
     * for v in node_versions {
     *     if v.lts.is_some() {
     *         println!("Found LTS {} version {}", v.lts.clone().unwrap(), v.version);
     *     }
     *     if arch == Arch::X86 {
     *         if os == Os::Linux
     *             || os == Os::Macos && v.version.satisfies(&"^10.0.0".parse().unwrap())
     *         {
     *             println!(
     *                 "32 bit versions are not available for macos and linux after version 10.0.0!"
     *             );
     *             return;
     *         }
     *     }
     *     println!("{:?}", v);
     * }
     */

    /*
     * let mut file = std::fs::File::create("file.tar.gz").unwrap();
     * let mut content = Cursor::new(response.await.unwrap().bytes().await.unwrap());
     * std::io::copy(&mut content, &mut file).unwrap();
     */

    /*
     * for v in value {
     *     println!("{:?}\n", v);
     * }
     */

    /*
     *     let r: Range = "12".parse().unwrap();
     *     let v: Version = "12.0".parse().unwrap();
     *
     *     println!("Got range {} and version {}", r, v);
     */

    // println!("In foo func for {:?}!", os);
    // println!("Version {}", r);
}

#[async_trait]
=======
                .unwrap();

            println!("file to download: '{}'", fname);
            let fname = dir.path().join(fname);
            File::create(fname).unwrap()
        };

        let content = response.bytes().await.unwrap();

        dest.write_all(&content).unwrap();
        println!("\n---\n");
    }
}

/*#[async_trait]
>>>>>>> e56eb72395529f1dd21931ce99282f6812000e6a
impl Command for Node {
    /// Display a help menu for the `volt add` command.
    fn help() -> String {
        format!(
            r#"volt {}

<<<<<<< HEAD
      Add a package to your project's dependencies.
      Usage: {} {} {} {}
      Options:

      {} {} Output the version number.
      {} {} Output verbose messages on internal operations.
      {} {} Adds package as a dev dependency
      {} {} Disable progress bar."#,
=======
            Manage NodeJS versions
            Usage: {} {} {} {}
            Options:

            {} {} Output the version number.
            {} {} Output verbose messages on internal operations.
            {} {} Adds package as a dev dependency
            {} {} Disable progress bar."#,
>>>>>>> e56eb72395529f1dd21931ce99282f6812000e6a
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

<<<<<<< HEAD
    /// Execute the `volt add` command
=======
    /// Execute the `volt node` command
>>>>>>> e56eb72395529f1dd21931ce99282f6812000e6a
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
<<<<<<< HEAD
        /*
         *         // Get input packages
         *         let mut packages = app.get_packages()?;
         *
         *         // Load the existing package.json file
         *         let (mut package_file, package_file_path) = PackageJson::open("package.json")?;
         *
         *         // Construct a path to the local and global lockfile.
         *         let lockfile_path = &app.lock_file_path;
         *
         *         let global_lockfile = &app.home_dir.join(".global.lock");
         *
         *         // Load local and global lockfiles.
         *         let mut lock_file =
         *             LockFile::load(lockfile_path).unwrap_or_else(|_| LockFile::new(lockfile_path));
         *
         *         let mut global_lock_file =
         *             LockFile::load(global_lockfile).unwrap_or_else(|_| LockFile::new(global_lockfile));
         *
         *         // Create progress bar for resolving dependencies.
         *
         *         let progress_bar = ProgressBar::new(packages.len() as u64);
         *
         *         progress_bar.set_style(
         *             ProgressStyle::default_bar()
         *                 .progress_chars(PROGRESS_CHARS)
         *                 .template(&format!(
         *                     "{} [{{bar:40.green/magenta}}] {{msg:.blue}}",
         *                     "Resolving Dependencies".bright_blue()
         *                 )),
         *         );
         *
         *         // Fetch npm data including hash to fetch dependencies
         *         let data = npm::get_versions(&packages).await?;
         *
         *         // Fetch pre-flattened dependency trees from the registry
         *         let (responses, elapsed) = fetch_dep_tree(&data, &progress_bar).await?;
         *
         *         let mut dependencies: HashMap<String, VoltPackage> = HashMap::new();
         *
         *         for res in responses.iter() {
         *             let current_version = res.versions.get(&res.version).unwrap();
         *             dependencies.extend(current_version.to_owned());
         *         }
         *
         *         progress_bar.finish_with_message("[OK]".bright_green().to_string());
         *
         *         print_elapsed(dependencies.len(), elapsed);
         *
         *         let mut dependencies: Vec<_> = dependencies
         *             .iter()
         *             .map(|(_name, object)| {
         *                 let mut lock_dependencies: Vec<String> = vec![];
         *
         *                 if let Some(peer_deps) = &object.peer_dependencies {
         *                     for dep in peer_deps {
         *                         if !crate::core::utils::check_peer_dependency(&dep) {
         *                             progress_bar.println(format!(
         *                                 "{}{} {} has unmet peer dependency {}",
         *                                 " warn ".black().bright_yellow(),
         *                                 ":",
         *                                 object.name.bright_cyan(),
         *                                 &dep.bright_yellow()
         *                             ));
         *                         }
         *                     }
         *                 }
         *
         *                 if let Some(dependencies) = &object.dependencies {
         *                     for dep in dependencies {
         *                         lock_dependencies.push(dep.to_string());
         *                     }
         *                 }
         *
         *                 let object_instance = object.clone();
         *
         *                 lock_file.dependencies.insert(
         *                     DependencyID(object_instance.name, object_instance.version),
         *                     DependencyLock {
         *                         name: object.name.clone(),
         *                         version: object.version.clone(),
         *                         tarball: object.tarball.clone(),
         *                         integrity: object.integrity.clone(),
         *                         dependencies: lock_dependencies.clone(),
         *                     },
         *                 );
         *
         *                 let second_instance = object.clone();
         *
         *                 global_lock_file.dependencies.insert(
         *                     DependencyID(second_instance.name, second_instance.version.to_owned()),
         *                     DependencyLock {
         *                         name: object.name.clone(),
         *                         version: object.version.clone(),
         *                         tarball: object.tarball.clone(),
         *                         integrity: object.integrity.clone(),
         *                         dependencies: lock_dependencies,
         *                     },
         *                 );
         *
         *                 object
         *             })
         *             .collect();
         *
         *         for dep in dependencies.iter() {
         *             for package in packages.iter_mut() {
         *                 if dep.name == package.name {
         *                     package.version = Some(dep.version.clone());
         *                 }
         *             }
         *         }
         *
         *         let progress_bar = ProgressBar::new(dependencies.len() as u64);
         *
         *         progress_bar.set_style(
         *             ProgressStyle::default_bar()
         *                 .progress_chars(PROGRESS_CHARS)
         *                 .template(&format!(
         *                     "{} [{{bar:40.green/magenta}}] {{msg:.blue}}",
         *                     "Installing Packages".bright_blue()
         *                 )),
         *         );
         *
         *         dependencies.dedup();
         *
         *         dependencies
         *             .into_iter()
         *             .map(|v| install_extract_package(&app, &v))
         *             .collect::<FuturesUnordered<_>>()
         *             .inspect(|_| progress_bar.inc(1))
         *             .try_collect::<()>()
         *             .await
         *             .unwrap();
         *
         *         progress_bar.finish();
         *
         *         for package in packages {
         *             package_file.add_dependency(package);
         *         }
         *
         *         package_file.save()?;
         *         global_lock_file.save()?;
         *         lock_file.save()?;
         *
         *         Ok(())
         */
    }
}
=======
    }
}*/
>>>>>>> e56eb72395529f1dd21931ce99282f6812000e6a
