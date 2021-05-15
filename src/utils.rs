/*
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

// Std Imports
use std::{borrow::Cow, path::PathBuf};
use std::{env, fs::create_dir};
use std::{env::temp_dir, fs::File};
use std::{
    fs::create_dir_all,
    io::{self, Write},
};
use std::{path::Path, process};

// Library Imports
use anyhow::{Context, Result};
use colored::Colorize;
use dirs::home_dir;
use flate2::read::GzDecoder;
use symlink::symlink_dir;
use tar::Archive;
use tokio::fs::remove_dir_all;

// Crate Level Imports
use crate::classes::package::Package;

#[cfg(windows)]
pub static PROGRESS_CHARS: &str = "=> ";

#[cfg(unix)]
pub static PROGRESS_CHARS: &str = "▰▰▱";

lazy_static! {
    pub static ref ERROR_TAG: String = "error".red().bold().to_string();
}

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

        let current_dir = env::current_dir().unwrap();
        let home_dir = home_dir().unwrap_or_else(|| current_dir.clone());
        let node_modules_dir = current_dir.join("node_modules");
        let volt_dir = home_dir.join(".volt");
        std::fs::create_dir_all(&volt_dir).ok();

        let lock_file_path = current_dir.join("volt.lock");

        let cli_args: Vec<_> = std::env::args().collect();
        let mut args: Vec<String> = Vec::new();
        let mut flags: Vec<String> = Vec::new();

        for arg in cli_args.into_iter().skip(2) {
            if arg.starts_with("--") || arg.starts_with("-") {
                flags.push(arg);
            } else {
                args.push(arg);
            }
        }

        App {
            args,
            flags,
            current_dir,
            home_dir,
            node_modules_dir,
            volt_dir,
            lock_file_path,
        }
    }

    pub fn has_flag(&self, flags: &[&str]) -> bool {
        self.flags
            .iter()
            .any(|flag| flags.iter().any(|search_flag| flag == search_flag))
    }
}

/// downloads tarball file from package
pub async fn download_tarball(_app: &App, package: &Package, version: &str) -> String {
    let name = &package
        .name
        .replace("/", "__")
        .replace("@", "")
        .replace(".", "_");
    let file_name = format!("{}@{}.tgz", name, version);
    let temp_dir = temp_dir();
    let path = temp_dir.join(file_name);
    let path_str = path.to_string_lossy().to_string();

    if path.exists() {
        return path_str;
    }

    let tarball = &package.versions[version]
        .dist
        .tarball
        .replace("https", "http");

    let mut response = reqwest::get(tarball).await.unwrap();

    // Placeholder buffer
    let mut file = File::create(path).unwrap();

    while let Some(chunk) = response.chunk().await.unwrap() {
        let _ = file.write(&*chunk);
    }

    path_str
}

pub fn get_basename<'a>(path: &'a str) -> Cow<'a, str> {
    let sep: char;
    if cfg!(windows) {
        sep = '\\';
    } else {
        sep = '/';
    }
    let mut pieces = path.rsplit(sep);

    match pieces.next() {
        Some(p) => p.into(),
        None => path.into(),
    }
}

pub async fn extract_tarball(
    file_path: &str,
    node_modules_dir: PathBuf,
    package: &Package,
) -> Result<()> {
    // Open tar file
    let tar_file = File::open(file_path).context("Unable to open tar file")?;
    let _ = create_dir_all(node_modules_dir.clone()); // not being created for me
    let home_dir_path = home_dir().unwrap();
    // Delete package from node_modules
    let node_modules_dep_path = node_modules_dir.join(&package.name);
    if node_modules_dep_path.exists() {
        remove_dir_all(&node_modules_dep_path).await.ok();
    }
    let home_dir_file_path = home_dir_path.join(".volt").join(package.name.clone());
    // Extract tar file
    let gz_decoder = GzDecoder::new(tar_file);
    let mut archive = Archive::new(gz_decoder);
    archive
        .unpack(format!("{}", home_dir_path.join(".volt").to_str().unwrap()))
        .context("Unable to unpack dependency")?;

    if home_dir_file_path.exists() {
        // do nothing
    } else {
        match std::fs::rename(
            format!(
                "{}",
                home_dir_path
                    .join(".volt")
                    .join("package")
                    .to_str()
                    .unwrap()
            ),
            format!("{}", home_dir_file_path.to_str().unwrap()),
        ) {
            Ok(_) => {}
            Err(err) => println!("error: {}", err.to_string().red().bold()),
        };
    }
    let f_path = home_dir_path.join(".volt").join(package.name.clone());
    create_symlink(f_path, node_modules_dep_path);
    Ok(())
}

/// Gets a config key from git using the git cli.
pub fn get_git_config(key: &str) -> io::Result<Option<String>> {
    process::Command::new("git")
        .arg("config")
        .arg("--get")
        .arg(key)
        .output()
        .map(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout[..output.stdout.len() - 1].to_vec()).ok()
            } else {
                None
            }
        })
}

// Windows Function
#[cfg(windows)]
fn enable_ansi_support() -> Result<(), u32> {
    // ref: https://docs.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences#EXAMPLE_OF_ENABLING_VIRTUAL_TERMINAL_PROCESSING @@ https://archive.is/L7wRJ#76%

    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;
    use winapi::um::consoleapi::{GetConsoleMode, SetConsoleMode};
    use winapi::um::errhandlingapi::GetLastError;
    use winapi::um::fileapi::{CreateFileW, OPEN_EXISTING};
    use winapi::um::handleapi::INVALID_HANDLE_VALUE;
    use winapi::um::winnt::{FILE_SHARE_WRITE, GENERIC_READ, GENERIC_WRITE};

    const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x0004;

    unsafe {
        // ref: https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew
        // Using `CreateFileW("CONOUT$", ...)` to retrieve the console handle works correctly even if STDOUT and/or STDERR are redirected
        let console_out_name: Vec<u16> =
            OsStr::new("CONOUT$").encode_wide().chain(once(0)).collect();
        let console_handle = CreateFileW(
            console_out_name.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_WRITE,
            null_mut(),
            OPEN_EXISTING,
            0,
            null_mut(),
        );
        if console_handle == INVALID_HANDLE_VALUE {
            return Err(GetLastError());
        }

        // ref: https://docs.microsoft.com/en-us/windows/console/getconsolemode
        let mut console_mode: u32 = 0;
        if 0 == GetConsoleMode(console_handle, &mut console_mode) {
            return Err(GetLastError());
        }

        // VT processing not already enabled?
        if console_mode & ENABLE_VIRTUAL_TERMINAL_PROCESSING == 0 {
            // https://docs.microsoft.com/en-us/windows/console/setconsolemode
            if 0 == SetConsoleMode(
                console_handle,
                console_mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING,
            ) {
                return Err(GetLastError());
            }
        }
    }

    return Ok(());
}

// Unix Function
#[cfg(unix)]
pub fn enable_ansi_support() -> Result<(), u32> {
    Ok(())
}

/// Create a symlink to a directory
pub fn create_symlink(path: PathBuf, destination: PathBuf) {
    if path.exists() {
        match symlink_dir(path, destination) {
            Ok(_) => {}
            Err(e) => {
                println!("{}: {}", "error".red().bold(), e.to_string().yellow());
            }
        };
    }
}
