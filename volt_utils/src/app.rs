use crate::enable_ansi_support;
use anyhow::Result;
use dirs::home_dir;
use sha1::{Digest, Sha1};
use std::{env, io, path::PathBuf};

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

        // Home Directory: /username or C:\Users\username
        let home_directory = home_dir().unwrap_or_else(|| current_directory.clone());

        // node_modules/
        let node_modules_directory = current_directory.join("node_modules");

        // Volt Global Directory: /username/.volt or C:\Users\username\.volt
        let volt_dir = home_directory.join(".volt");

        // Create volt directory if it doesn't exist
        std::fs::create_dir_all(&volt_dir).ok();

        // ./volt.lock
        let lock_file_path = current_directory.join("volt.lock");

        let cli_args: Vec<_> = std::env::args().collect();

        let mut refined_args: Vec<String> = Vec::new();

        let mut flags: Vec<String> = Vec::new();

        for arg in cli_args.into_iter().skip(1) {
            if arg.starts_with("--") || arg.starts_with('-') {
                flags.push(arg);
            } else {
                refined_args.push(arg);
            }
        }

        App {
            current_dir: current_directory,
            home_dir: home_directory,
            node_modules_dir: node_modules_directory,
            volt_dir,
            lock_file_path,
            args: refined_args,
            flags,
        }
    }

    /// Check if the app arguments contain the flags specified
    pub fn has_flag(&self, flags: &[&str]) -> bool {
        self.flags
            .iter()
            .any(|flag| flags.iter().any(|search_flag| flag == search_flag))
    }

    pub fn calc_hash(data: &bytes::Bytes) -> Result<String> {
        let mut hasher = Sha1::new();
        io::copy(&mut &**data, &mut hasher)?;

        Ok(format!("{:x}", hasher.finalize()))
    }
}
