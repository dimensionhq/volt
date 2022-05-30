use std::fmt::Display;

use async_trait::async_trait;
use clap::CommandFactory;
use clap::{ErrorKind, Parser};
use colored::Colorize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use miette::Result;
use node_semver::{Range, Version};
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

use crate::{
    cli::{VoltCommand, VoltConfig},
    commands::node::NodeVersion,
};

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
        write!(f, "{s}")
    }
}

const ARCH: Arch = if cfg!(target_arch = "X86") {
    Arch::X86
} else if cfg!(target_arch = "x86_64") {
    Arch::X64
} else {
    Arch::Unknown
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
        if self.versions.is_empty() {
            let mut cmd = NodeInstall::command();
            cmd.error(
                ErrorKind::ArgumentConflict,
                "Must have at least one version",
            )
            .exit();
        }

        tracing::debug!("On platform '{}' and arch '{}'", PLATFORM, ARCH);
        let dir = tempdir().unwrap();
        tracing::debug!("Temp dir is {:?}", dir);

        let mirror = "https://nodejs.org/dist";

        // Deserialize all available NodeJS versions
        let node_versions: Vec<NodeVersion> = reqwest::get(format!("{}/index.json", mirror))
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        let node_path = {
            let datadir = dirs::data_dir().unwrap().join("volt").join("node");
            if !datadir.exists() {
                std::fs::create_dir_all(&datadir).unwrap();
            }
            datadir
        };

        let mut validversions = vec![];
        let download_url = format!("{}/", mirror);

        // TODO: Maybe suggest the closest available version if not found?
        // Validate the list of requested versions to install
        for v in &self.versions {
            let current_version: Option<Version> = if let Ok(ver) = v.parse() {
                if cfg!(all(unix, target_arch = "X86")) && ver >= Version::parse("10.0.0").unwrap()
                {
                    println!("32 bit versions are not available for MacOS and Linux after version 10.0.0!");
                    continue;
                }

                let mut found = false;
                for n in &node_versions {
                    if *v == n.version.to_string() {
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
                // TODO: Not a valid version
                println!("Invalid version: {}!", v.truecolor(255, 0, 0));
                std::process::exit(1);
            };

            if let Some(version) = current_version {
                validversions.push(version)
            } else {
                println!("Invalid version: {}!", v.truecolor(255, 0, 0));
                std::process::exit(1);
            }
        }

        let mb = MultiProgress::new();

        // Spawn a thread to download each version
        let handles: Vec<_> = validversions
            .clone()
            .into_iter()
            .map(|i| {
                let download_url = format!("{download_url}v{i}/node-v{i}-{PLATFORM}-{ARCH}.tar.xz");

                let pb = mb.add(ProgressBar::new_spinner().with_style(
                    ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}"),
                ));

                let handle = tokio::runtime::Handle::current();

                let node_path = node_path.clone();

                let dir = dir.path().to_owned();
                handle.spawn_blocking(move || {
                    if node_path.join(&i.to_string()).exists() {
                        pb.set_message(format!(
                            "{:8} {}",
                            i.to_string().truecolor(0, 255, 0),
                            "Already Installed ✓"
                        ));
                        pb.finish();
                        return;
                    }

                    pb.set_message(format!(
                        "{:8} {:10}",
                        i.to_string().truecolor(125, 125, 125),
                        String::from("Installing")
                    ));

                    pb.enable_steady_tick(10);

                    let response = reqwest::blocking::get(&download_url).unwrap();
                    let content = response.bytes().unwrap();

                    #[cfg(target_family = "unix")]
                    {
                        // Path to write the decompressed tarball to
                        let fname = download_url.split('/').last().unwrap().to_string();
                        let tarpath = dir.join(&fname.strip_suffix(".xz").unwrap());

                        // Decompress the tarball
                        let mut tarball = File::create(&tarpath).unwrap();
                        tarball
                            .write_all(&lzma::decompress(&content).unwrap())
                            .unwrap();

                        // Make sure the first file handle is closed
                        drop(tarball);

                        // Have to reopen it for reading, File::create() opens for write only
                        let tarball = File::open(&tarpath).unwrap();

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
                        let to = node_path.join(&i.to_string());

                        // TODO: Find a way to handle this error correctly?
                        // Rename the folder from the default set by the tarball
                        // to just the version number
                        let _ = std::fs::rename(from, to);
                    }

                    pb.set_message(format!(
                        "{:8} {:10}",
                        i.to_string().truecolor(0, 255, 0),
                        "Installed ✓"
                    ));
                    pb.finish();
                })
            })
            .collect();

        futures::future::join_all(handles).await;

        Ok(())
    }
}
