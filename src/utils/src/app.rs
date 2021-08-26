use crate::{enable_ansi_support, errors::VoltError};
use dirs::home_dir;
use miette::DiagnosticResult;
use sha1::Digest;
use sha2::Sha512;
use ssri::{Algorithm, Integrity};
use std::{env, path::PathBuf};

#[derive(Debug, PartialEq)]
pub enum AppFlag {
    Help,
    Version,
    Yes,
    Depth,
    Verbose,
    NoProgress,
    Dev,
}

impl AppFlag {
    pub fn get(arg: &String) -> Option<AppFlag> {
        let mut flag = arg.to_string();

        // --verbose -> verbose
        while flag.starts_with("-") {
            flag.remove(0);
        }

        match flag.to_lowercase().as_str() {
            "help" => Some(AppFlag::Help),
            "h" => Some(AppFlag::Help),
            "version" => Some(AppFlag::Version),
            "yes" => Some(AppFlag::Yes),
            "y" => Some(AppFlag::Yes),
            "depth" => Some(AppFlag::Depth),
            "verbose" => Some(AppFlag::Verbose),
            "no-progress" => Some(AppFlag::NoProgress),
            &_ => None,
        }
    }
}

#[derive(Debug)]
pub struct App {
    pub current_dir: PathBuf,
    pub home_dir: PathBuf,
    pub node_modules_dir: PathBuf,
    pub volt_dir: PathBuf,
    pub lock_file_path: PathBuf,
    pub args: Vec<String>,
    pub flags: Vec<AppFlag>,
    pub unknown_flags: Vec<String>,
}

impl App {
    pub fn initialize() -> DiagnosticResult<App> {
        enable_ansi_support().unwrap();

        // Current Directory
        let current_directory = env::current_dir().map_err(|_e| VoltError::EnvironmentError {
            env: "CURRENT_DIRECTORY".to_string(),
        })?;

        // Home Directory: /username or C:\Users\username
        let home_directory = home_dir().ok_or(VoltError::EnvironmentError {
            env: "HOME".to_string(),
        })?;

        // node_modules/
        let node_modules_directory = current_directory.join("node_modules");

        // Volt Global Directory: /username/.volt or C:\Users\username\.volt
        let volt_dir = home_directory.join(".volt");

        // Create volt directory if it doesn't exist
        std::fs::create_dir_all(&volt_dir).map_err(VoltError::CreateDirError)?;

        // ./volt.lock
        let lock_file_path = current_directory.join("volt.lock");

        let cli_args: Vec<String> = std::env::args().collect();

        let mut refined_args: Vec<String> = Vec::new();

        let mut flags: Vec<AppFlag> = Vec::new();
        let mut unknown_flags: Vec<String> = Vec::new();

        for arg in cli_args.into_iter().skip(1) {
            if arg.starts_with("--") || arg.starts_with('-') {
                match AppFlag::get(&arg) {
                    Some(flag) => flags.push(flag),
                    None => unknown_flags.push(arg),
                }
            } else {
                refined_args.push(arg);
            }
        }

        Ok(App {
            current_dir: current_directory,
            home_dir: home_directory,
            node_modules_dir: node_modules_directory,
            volt_dir,
            lock_file_path,
            args: refined_args,
            flags,
            unknown_flags,
        })
    }

    /// Check if the app arguments contain the flags specified
    pub fn has_flag(&self, flag: AppFlag) -> bool {
        self.flags.contains(&flag)
    }

    pub fn calc_hash(data: &bytes::Bytes, algorithm: Algorithm) -> DiagnosticResult<String> {
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
