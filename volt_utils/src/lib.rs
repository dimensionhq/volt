pub mod app;
pub mod package;
pub mod voltapi;
use std::sync::Arc;

use anyhow::Context;
use chttp::{self, ResponseExt};
use colored::Colorize;
use flate2::read::GzDecoder;
use futures_util::stream::FuturesUnordered;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::borrow::Cow;
use std::fs::{create_dir, remove_dir_all};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::{env::temp_dir, fs::File};
use tar::Archive;
use tokio::fs::create_dir_all;
use tokio::fs::hard_link;
use walkdir::WalkDir;

use anyhow::Error;
use anyhow::Result;
use app::App;
use lazy_static::lazy_static;
use package::Package;
use voltapi::{VoltPackage, VoltResponse};

#[cfg(windows)]
pub static PROGRESS_CHARS: &str = "=> ";

#[cfg(target_os = "linux")]
pub static PROGRESS_CHARS: &str = "▰▰▱";

lazy_static! {
    pub static ref ERROR_TAG: String = "error".red().bold().to_string();
}

// async fn get_dependencies_recursive(
//     app: Arc<App>,
//     packages: &std::collections::HashMap<String, VoltPackage>,
// ) {
//     for package in packages.iter() {
//         install_extract_package(app.clone(), package.1)
//             .await
//             .unwrap();
//     }
// }

pub fn create_dependency_links(
    _app: Arc<App>,
    packages: std::collections::HashMap<String, VoltPackage>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'static>> {
    Box::pin(async move {
        let user_profile;
        let volt_dir_loc;

        if cfg!(target_os = "windows") {
            user_profile = std::env::var("USERPROFILE").unwrap();
            volt_dir_loc = format!(r"{}\.volt", user_profile);
        } else {
            user_profile = std::env::var("HOME").unwrap();
            volt_dir_loc = format!(r"{}/.volt", user_profile);
        }

        let mut workers = FuturesUnordered::new();

        for package in packages {
            let pkg_clone = package.clone();
            let volt_directory_location = volt_dir_loc.clone();
            workers.push(async move {
                // Hardlink Files
                hardlink_files(format!(r"{}\{}", volt_directory_location, pkg_clone.1.name)).await;
            });
        }

        while workers.next().await.is_some() {}
        Ok(())
    })
}

// Get response from volt CDN
pub async fn get_volt_response(package_name: String) -> VoltResponse {
    let time = std::time::Instant::now();
    let response = chttp::get_async(format!("http://volt-api.b-cdn.net/{}.json", package_name))
        .await
        .unwrap_or_else(|_| {
            println!("{}: package does not exist", "error".bright_red(),);
            std::process::exit(1);
        })
        .text_async()
        .await
        .unwrap_or_else(|_| {
            println!("{}: package does not exist", "error".bright_red());
            std::process::exit(1);
        });

    // println!("Response Time: {:?}", time.elapsed());

    serde_json::from_str::<VoltResponse>(&response).unwrap_or_else(|_| {
        println!(
            "{}: failed to parse response from server",
            "error".bright_red()
        );
        std::process::exit(1);
    })
}

pub async fn hardlink_files(src: String) {
    let volt_dir_loc;
    let user_profile;

    if cfg!(target_os = "windows") {
        user_profile = std::env::var("USERPROFILE").unwrap();
        volt_dir_loc = format!(r"{}\.volt", user_profile).replace(r"\", "/");
    } else {
        user_profile = std::env::var("HOME").unwrap();
        volt_dir_loc = format!(r"{}/.volt", user_profile);
    }

    for entry in WalkDir::new(src) {
        let entry = entry.unwrap();
        if !entry.path().is_dir() {
            let file_name = &entry.path().file_name().unwrap().to_str().unwrap();
            let path = format!("{}", &entry.path().display())
                .replace(r"\", "/")
                .replace(&volt_dir_loc, "");

            create_dir_all(format!(
                "node_modules/{}",
                &path.trim_end_matches(file_name)
            ))
            .await
            .unwrap();

            hard_link(
                format!("{}{}", volt_dir_loc, &path),
                format!("node_modules{}", &path),
            )
            .await
            .unwrap();
        }
    }
}

/// downloads tarball file from package
pub async fn download_tarball(app: &App, package: &VoltPackage) -> Result<String> {
    let name = &package.name.replace("/", "__");
    let file_name = format!("{}@{}.tgz", name, package.version);
    let temp_dir = temp_dir();

    if !Path::new(&temp_dir.join("volt")).exists() {
        std::fs::create_dir(Path::new(&temp_dir.join("volt")))?;
    }

    if name.starts_with('@') && name.contains("__") {
        let package_dir_loc;

        if cfg!(target_os = "windows") {
            // Check if C:\Users\username\.volt exists
            package_dir_loc = format!(
                r"{}\.volt\{}",
                std::env::var("USERPROFILE").unwrap(),
                name.split("__").collect::<Vec<&str>>()[0]
            );
        } else {
            // Check if ~/.volt\packagename exists
            package_dir_loc = format!(
                r"{}\.volt\{}",
                std::env::var("HOME").unwrap(),
                name.split("__").collect::<Vec<&str>>()[0]
            );
        }

        if !Path::new(&package_dir_loc).exists() {
            create_dir_all(&package_dir_loc).await.unwrap();
        }
    }

    let path;

    if cfg!(target_os = "windows") {
        path = temp_dir.join(format!(r"volt\{}", file_name));
    } else {
        path = temp_dir.join(format!(r"volt/{}", file_name));
    }

    let path_str = path.to_string_lossy().to_string();

    if path.exists() {
        let bytes = std::fs::read(path_str.clone()).unwrap();

        if let Ok(hash) = App::calc_hash(&bytes::Bytes::from(bytes)) {
            // File exists, make sure it's not corrupted
            if hash == package.sha1 {
                println!("Valid Hash!");
                return Ok(path_str);
            }
        }
    }

    let tarball = package.tarball.replace("https", "http");

    let res = reqwest::get(tarball).await.unwrap();

    let bytes: bytes::Bytes = res.bytes().await.unwrap();

    App::calc_hash(&bytes)?;

    create_dir_all(&app.node_modules_dir).await?;

    // Delete package from node_modules
    let node_modules_dep_path = app.node_modules_dir.join(&package.name);

    if node_modules_dep_path.exists() {
        remove_dir_all(&node_modules_dep_path)?;
    }

    let loc = format!(r"{}\{}", &app.volt_dir.to_str().unwrap(), package.name);

    let path = Path::new(&loc);

    if !path.exists() {
        // Extract tar file
        let gz_decoder = GzDecoder::new(&*bytes);
        let mut archive = Archive::new(gz_decoder);

        let mut vlt_dir = PathBuf::from(&app.volt_dir);

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
                create_dir_all(&parent).await?;
            }
        }
    }

    // extract now
    Ok(path_str)
}

pub async fn download_tarball_create(
    _app: &App,
    package: &Package,
    name: &str,
) -> Result<String, Error> {
    let file_name = format!("{}-{}.tgz", name, package.dist_tags.latest);
    let temp_dir = temp_dir();

    if !Path::new(&temp_dir.join("volt")).exists() {
        std::fs::create_dir(Path::new(&temp_dir.join("volt")))?;
    }

    if name.starts_with('@') && name.contains("__") {
        let package_dir_loc;

        if cfg!(target_os = "windows") {
            // Check if C:\Users\username\.volt exists
            package_dir_loc = format!(
                r"{}\.volt\{}",
                std::env::var("USERPROFILE").unwrap(),
                name.split("__").collect::<Vec<&str>>()[0]
            );
        } else {
            // Check if ~/.volt\packagename exists
            package_dir_loc = format!(
                r"{}\.volt\{}",
                std::env::var("HOME").unwrap(),
                name.split("__").collect::<Vec<&str>>()[0]
            );
        }

        if !Path::new(&package_dir_loc).exists() {
            create_dir_all(&package_dir_loc).await.unwrap();
        }
    }

    let path;

    if cfg!(target_os = "windows") {
        path = temp_dir.join(format!(r"volt\{}", file_name));
    } else {
        path = temp_dir.join(format!(r"volt/{}", file_name));
    }

    let path_str = path.to_string_lossy().to_string();
    let package_version = package.versions.get(&package.dist_tags.latest).unwrap();

    let bytes = std::fs::read(path_str.clone()).unwrap();

    // Corrupt tar files may cause issues
    if let Ok(hash) = App::calc_hash(&bytes::Bytes::from(bytes)) {
        // File exists, make sure it's not corrupted
        if hash
            == package
                .versions
                .get(&package.dist_tags.latest)
                .unwrap()
                .dist
                .shasum
        {
            return Ok(path_str);
        }
    }

    let tarball = package_version.dist.tarball.replace("https", "http");

    let res = reqwest::get(tarball).await.unwrap();

    let bytes = res.bytes().await.unwrap();

    App::calc_hash(&bytes)?;

    Ok(path_str)
}
pub fn get_basename(path: &'_ str) -> Cow<'_, str> {
    let sep: char;
    if cfg!(target_os = "windows") {
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
pub fn enable_ansi_support() -> Result<(), u32> {
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

    Ok(())
}

/// Create a junction / hard symlink to a directory
#[cfg(windows)]
pub fn create_symlink(original: String, link: String) -> Result<()> {
    println!("symlinking from {} to {}", original, link);
    junction::create(original, link)?;
    Ok(())
}

#[cfg(windows)]
pub fn generate_script(package: &VoltPackage) {
    if !Path::new("node_modules/scripts").exists() {
        create_dir("node_modules/scripts").unwrap();
    }

    if package.bin.is_some() {
        let bin = package.clone().bin.unwrap();
        let k = bin.keys().next().unwrap();
        let v = bin.values().next().unwrap();

        let user_profile = std::env::var("USERPROFILE").unwrap();

        let volt_path = format!("{}/.volt", user_profile);
        let command = format!(
            r#"
@IF EXIST "%~dp0\node.exe" (
    "%~dp0\node.exe"  "%~dp0\..\{}\{}" %*
) ELSE (
    @SETLOCAL
    @SET PATHEXT=%PATHEXT:;.JS;=;%
    node  "%~dp0\..\{}\{}" %*
)"#,
            k, v, k, v
        )
        .replace(r"%~dp0\..", &volt_path);

        let mut f = File::create(format!(r"node_modules/scripts/{}.cmd", k)).unwrap();
        f.write_all(command.as_bytes()).unwrap();
    }
}

// Unix functions
#[cfg(unix)]
pub fn enable_ansi_support() -> Result<(), u32> {
    Ok(())
}
#[cfg(unix)]

pub fn generate_script(package: &VoltPackage) {}

pub fn check_peer_dependency(_package_name: &str) -> bool {
    false
}

pub async fn install_extract_package(app: Arc<App>, package: &VoltPackage) -> Result<()> {
    let pb = ProgressBar::new(0);
    let text = format!("{}", "Installing Packages".bright_cyan());

    pb.set_style(
        ProgressStyle::default_spinner()
            .template(("{spinner:.green}".to_string() + format!(" {}", text).as_str()).as_str())
            .tick_strings(&["┤", "┘", "┴", "└", "├", "┌", "┬", "┐"]),
    );

    download_tarball(&app, &package).await?;

    generate_script(package);

    Ok(())
}
