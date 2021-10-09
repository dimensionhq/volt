use crate::{
    commands::add::PackageInfo,
    core::utils::{enable_ansi_support, errors::VoltError},
};
use clap::ArgMatches;
use dirs::home_dir;
use miette::Result;
use sha1::Digest;
use sha2::Sha512;
use ssri::{Algorithm, Integrity};
use std::{env, path::PathBuf};

use super::npm::parse_versions;

#[derive(Debug)]
pub struct App {
    pub current_dir: PathBuf,
    pub home_dir: PathBuf,
    pub node_modules_dir: PathBuf,
    pub volt_dir: PathBuf,
    pub lock_file_path: PathBuf,
    pub args: ArgMatches,
}

impl App {
    pub fn initialize(args: &ArgMatches) -> Result<App> {
        enable_ansi_support().unwrap();

        // Current Directory
        let current_directory = env::current_dir().map_err(|e| VoltError::EnvironmentError {
            env: "CURRENT_DIRECTORY".to_string(),
            source: e,
        })?;

        // Home Directory: /username or C:\Users\username
        let home_directory = home_dir().ok_or(VoltError::GetHomeDirError)?;

        // node_modules/
        let node_modules_directory = current_directory.join("node_modules");

        // Volt Global Directory: /username/.volt or C:\Users\username\.volt
        let volt_dir = home_directory.join(".volt");

        // Create volt directory if it doesn't exist
        std::fs::create_dir_all(&volt_dir).map_err(VoltError::CreateDirError)?;

        // ./volt.lock
        let lock_file_path = current_directory.join("volt.lock");

        Ok(App {
            current_dir: current_directory,
            home_dir: home_directory,
            node_modules_dir: node_modules_directory,
            volt_dir,
            lock_file_path,
            args: args.to_owned(),
        })
    }

    /// Retrieve packages passed in
    pub fn get_packages(&self) -> Result<Vec<PackageInfo>> {
        let mut args = self
            .args
            .values_of("package-names")
            .unwrap()
            .map(|v| v.to_string())
            .collect::<Vec<String>>();

        args.dedup();

        parse_versions(&args)
    }

    /// Check if the app arguments contain the flags specified
    pub fn has_flag(&self, flag: &str) -> bool {
        self.args.is_present(flag)
    }

    /// Calculate the hash of a tarball
    ///
    /// ## Examples
    /// ```rs
    /// calc_hash(bytes::Bytes::new(), ssri::Algorithm::Sha1)?;
    /// ```
    /// ## Returns
    /// * Result<String>
    pub fn calc_hash(data: &bytes::Bytes, algorithm: Algorithm) -> Result<String> {
        match algorithm {
            Algorithm::Sha1 => {
                let mut hasher = sha1::Sha1::new();
                std::io::copy(&mut &**data, &mut hasher).map_err(VoltError::HasherCopyError)?;

                let integrity: Integrity = format!(
                    "sha1-{}",
                    base64::encode(format!("{:x}", hasher.clone().finalize()))
                )
                .parse()
                .map_err(|_| VoltError::HashParseError {
                    hash: format!(
                        "sha1-{}",
                        base64::encode(format!("{:x}", hasher.clone().finalize()))
                    ),
                })?;

                let hash = integrity
                    .hashes
                    .into_iter()
                    .find(|h| h.algorithm == algorithm)
                    .map(|h| Integrity { hashes: vec![h] })
                    .map(|i| i.to_hex().1)
                    .unwrap();

                return Ok(format!("sha1-{}", hash));
            }
            Algorithm::Sha512 => {
                let mut hasher = Sha512::new();
                std::io::copy(&mut &**data, &mut hasher).map_err(VoltError::HasherCopyError)?;
                return Ok(format!("sha512-{:x}", hasher.finalize()));
            }
            _ => Ok(String::new()),
        }
    }
}
