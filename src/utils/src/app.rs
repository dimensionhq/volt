use crate::enable_ansi_support;
use anyhow::{Context, Result};
use dirs::home_dir;
use sha1::{Digest, Sha1};
use std::{env, io, path::PathBuf};

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

        while flag.starts_with("-") {
            flag.remove(0);
        }

        match flag.to_lowercase().as_str() {
            "help" => Some(AppFlag::Help),
            "h" => Some(AppFlag::Help),
            "version" => Some(AppFlag::Version),
            "yes" => Some(AppFlag::Yes),
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
    pub fn initialize() -> Result<App> {
        enable_ansi_support().unwrap();

        // Current Directory
        let current_directory = env::current_dir().unwrap();

        // Home Directory: /username or C:\Users\username
        let home_directory = home_dir().context("Failed to detect $HOME environment variable.")?;

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

    pub fn calc_hash(data: &bytes::Bytes) -> Result<String> {
        let mut hasher = Sha1::new();
        io::copy(&mut &**data, &mut hasher)?;

        Ok(format!("{:x}", hasher.finalize()))
    }
}
