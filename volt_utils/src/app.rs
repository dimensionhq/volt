use crate::enable_ansi_support;
use anyhow::{Context, Result};
use colored::Colorize;
use dirs::home_dir;
use flate2::read::GzDecoder;
use sha1::{Digest, Sha1};
use std::{
    env,
    fs::{create_dir_all, remove_dir_all, File},
    io::{self},
    path::{Path, PathBuf},
};
use tar::Archive;

use super::voltapi::*;

#[derive(Debug)]
pub struct App {
    pub current_dir: PathBuf,
    pub home_dir: PathBuf,
    pub node_modules_dir: PathBuf,
    pub volt_dir: PathBuf,
    pub lock_file_path: PathBuf,
    pub args: Vec<String>,
    pub flags: Vec<String>,
}

impl App {
    pub fn initialize() -> Self {
        enable_ansi_support().unwrap();

        // Current Directory
        let current_directory = env::current_dir().unwrap();

        // 
        let home_directory = home_dir().unwrap_or_else(|| current_directory.clone());
        let node_modules_dir = current_directory.join("node_modules");
        let volt_dir = home_directory.join(".volt");
        std::fs::create_dir_all(&volt_dir).ok();

        let lock_file_path = current_directory.join("volt.lock");

        let cli_args: Vec<_> = std::env::args().collect();
        let mut args: Vec<String> = Vec::new();
        let mut flags: Vec<String> = Vec::new();

        for arg in cli_args.into_iter().skip(1) {
            if arg.starts_with("--") || arg.starts_with('-') {
                flags.push(arg);
            } else {
                args.push(arg);
            }
        }

        App {
            current_dir: current_directory,
            home_dir: home_directory,
            node_modules_dir,
            volt_dir,
            lock_file_path,
            args,
            flags,
        }
    }

    pub fn has_flag(&self, flags: &[&str]) -> bool {
        self.flags
            .iter()
            .any(|flag| flags.iter().any(|search_flag| flag == search_flag))
    }

    pub async fn extract_tarball(&self, file_path: &str, package: &VoltPackage) -> Result<()> {
        // Open tar file
        let tar_file = File::open(file_path).context("Unable to open tar file")?;

        create_dir_all(&self.node_modules_dir)?;

        // Delete package from node_modules
        let node_modules_dep_path = self.node_modules_dir.join(&package.name);

        if node_modules_dep_path.exists() {
            remove_dir_all(&node_modules_dep_path)?;
        }

        let loc = format!(r"{}\{}", &self.volt_dir.to_str().unwrap(), package.name);

        let path = Path::new(&loc);

        if !path.exists() {
            // Extract tar file
            let gz_decoder = GzDecoder::new(tar_file);
            let mut archive = Archive::new(gz_decoder);

            let mut vlt_dir = PathBuf::from(&self.volt_dir);

            if package.clone().name.starts_with('@') && package.clone().name.contains(r"/") {
                if cfg!(target_os = "windows") {
                    let name = package.clone().name.replace(r"/", r"\");

                    let split = name.split(r"\").collect::<Vec<&str>>();

                    vlt_dir = vlt_dir.join(split[0]);
                } else {
                    let name = package.clone().name;

                    let split = name.split('/').collect::<Vec<&str>>();

                    vlt_dir = vlt_dir.join(split[0]);
                }
            }

            archive
                .unpack(&vlt_dir)
                .context("Unable to unpack dependency")?;

            if cfg!(target_os = "windows") {
                let mut idx = 0;
                let name = package.clone().name;

                let split = name.split('/').collect::<Vec<&str>>();

                if package.clone().name.contains('@') && package.clone().name.contains('/') {
                    idx = 1;
                }

                if Path::new(format!(r"{}\package", &vlt_dir.to_str().unwrap()).as_str()).exists() {
                    std::fs::rename(
                        format!(r"{}\package", &vlt_dir.to_str().unwrap()),
                        format!(r"{}\{}", &vlt_dir.to_str().unwrap(), split[idx]),
                    )
                    .context("failed to rename dependency folder")
                    .unwrap_or_else(|e| println!("{} {}", "error".bright_red(), e));
                }
            } else {
                std::fs::rename(
                    format!(r"{}/package", &vlt_dir.to_str().unwrap()),
                    format!(
                        r"{}/{}",
                        &vlt_dir.to_str().unwrap(),
                        package.name.replace("/", "__").replace("@", "")
                    ),
                )
                .context("Failed to unpack dependency folder")
                .unwrap_or_else(|e| println!("{} {}", "error".bright_red(), e));
            }
            if let Some(parent) = node_modules_dep_path.parent() {
                if !parent.exists() {
                    create_dir_all(&parent)?;
                }
            }
        }

        Ok(())
    }

    pub fn calc_hash(data: &bytes::Bytes) -> Result<String> {
        let mut hasher = Sha1::new();
        io::copy(&mut &**data, &mut hasher)?;

        Ok(format!("{:x}", hasher.finalize()))
    }
}
