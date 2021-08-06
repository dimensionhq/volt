pub mod app;
pub mod constants;
pub mod helper;
pub mod package;
pub mod volt_api;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Error;
use anyhow::Result;
use app::App;
use chttp::http::StatusCode;
use chttp::ResponseExt;
use colored::Colorize;
use flate2::read::GzDecoder;
use futures_util::stream::FuturesUnordered;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use package::Package;
use rand::prelude::SliceRandom;
use std::borrow::Cow;
use std::env::temp_dir;
use std::fs::{remove_dir_all, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::Arc;
use tar::Archive;
use tokio::fs::create_dir_all;
use tokio::fs::hard_link;
use volt_api::{VoltPackage, VoltResponse};
use walkdir::WalkDir;

use crate::constants::MAX_RETRIES;
use crate::helper::ResultLogErrorExt;

// Get response from volt CDN
pub async fn get_volt_response(package_name: String) -> Result<VoltResponse> {
    let mut retries = 0;

    loop {
        let mut response = chttp::get_async(format!(
            "http://push-2105.5centscdn.com/{}.bin",
            package_name
        ))
        .await
        .with_context(|| format!("failed to fetch {}", package_name.bright_yellow().bold()))?;

        match response.status_mut() {
            &mut StatusCode::OK => {
                let text = response.text_async().await.context(format!(
                    "failed to deserialize response for {}",
                    package_name.bright_yellow().bold()
                ))?;

                return Ok(serde_json::from_str(&text)?);
            }
            &mut StatusCode::NOT_FOUND => {
                if retries == MAX_RETRIES {
                    return Err(anyhow!(
                        "GET {} - {}\n\n{} was not found on the volt registry, or you don't have the permission to request it.\n",
                        format!("http://registry.voltpkg.com/{}", package_name),
                        format!("Not Found ({})", "404".bright_yellow().bold()),
                        package_name,
                    ));
                }
            }
            _ => {
                if retries == MAX_RETRIES {
                    return Err(anyhow!(
                        "{} {}: Not Found - {}\n\n{} was not found on the volt registry, or you don't have the permission to request it.",
                        "GET".bright_green(),
                        format!("http://registry.voltpkg.com/{}", package_name).underline(),
                        response.status().as_str(),
                        package_name
                    ));
                }
            }
        }

        retries += 1;
    }
}

pub async fn get_volt_response_multi(packages: Vec<String>) -> Vec<Result<VoltResponse>> {
    packages
        .into_iter()
        .map(get_volt_response)
        .collect::<FuturesUnordered<_>>()
        .collect::<Vec<Result<VoltResponse>>>()
        .await
}

#[cfg(windows)]
pub async fn hardlink_files(app: Arc<App>, src: PathBuf) {
    println!("{}", app.volt_dir.display());

    // for entry in WalkDir::new(src) {
    //     let entry = entry.unwrap();

    //     if !entry.path().is_dir() {
    //         // index.js
    //         let file_name = &entry.path().file_name().unwrap().to_str().unwrap();

    //         // lib/index.js
    //         let path = format!("{}", &entry.path().display())
    //             .replace(r"\", "/")
    //             .replace(&volt_directory, "");

    //         // node_modules/lib
    //         create_dir_all(format!(
    //             "node_modules/{}",
    //             &path
    //                 .replace(
    //                     format!("{}", &app.volt_dir.display())
    //                         .replace(r"\", "/")
    //                         .as_str(),
    //                     ""
    //                 )
    //                 .trim_end_matches(file_name)
    //         ))
    //         .await
    //         .unwrap();

    //         // ~/.volt/package/lib/index.js -> node_modules/package/lib/index.js
    //         if !Path::new(&format!(
    //             "node_modules{}",
    //             &path.replace(
    //                 format!("{}", &app.volt_dir.display())
    //                     .replace(r"\", "/")
    //                     .as_str(),
    //                 ""
    //             )
    //         ))
    //         .exists()
    //         {
    //             hard_link(
    //                 format!("{}", &path),
    //                 format!(
    //                     "node_modules{}",
    //                     &path.replace(
    //                         format!("{}", &app.volt_dir.display())
    //                             .replace(r"\", "/")
    //                             .as_str(),
    //                         ""
    //                     )
    //                 ),
    //             )
    //             .await
    //             .unwrap_or_else(|_| {
    //                 0;
    //             });
    //         }
    //     }
    // }
}

#[cfg(unix)]
pub async fn hardlink_files(app: Arc<App>, src: PathBuf) {
    let mut src = src;
    let volt_directory = format!("{}", app.volt_dir.display());

    if !cfg!(target_os = "windows") {
        src = src.replace(r"\", "/");
    }

    for entry in WalkDir::new(src) {
        let entry = entry.unwrap();

        if !entry.path().is_dir() {
            // index.js
            let file_name = &entry.path().file_name().unwrap().to_str().unwrap();

            // lib/index.js
            let path = format!("{}", &entry.path().display())
                .replace(r"\", "/")
                .replace(&volt_directory, "");

            // node_modules/lib
            create_dir_all(format!(
                "node_modules/{}",
                &path
                    .replace(
                        format!("{}", &app.volt_dir.display())
                            .replace(r"\", "/")
                            .as_str(),
                        ""
                    )
                    .trim_end_matches(file_name)
            ))
            .await
            .unwrap();

            // ~/.volt/package/lib/index.js -> node_modules/package/lib/index.js
            if !Path::new(&format!(
                "node_modules{}",
                &path.replace(
                    format!("{}", &app.volt_dir.display())
                        .replace(r"\", "/")
                        .as_str(),
                    ""
                )
            ))
            .exists()
            {
                hard_link(
                    format!("{}/.volt/{}", std::env::var("HOME").unwrap(), path),
                    format!(
                        "{}/node_modules{}",
                        std::env::current_dir().unwrap().to_string_lossy(),
                        &path.replace(
                            format!("{}", &app.volt_dir.display())
                                .replace(r"\", "/")
                                .as_str(),
                            ""
                        )
                    ),
                )
                .await
                .unwrap_or_else(|e| {
                    panic!(
                        "{:#?}",
                        (
                            format!("{}", &path),
                            format!(
                                "node_modules{}",
                                &path.replace(
                                    format!("{}", &app.volt_dir.display())
                                        .replace(r"\", "/")
                                        .as_str(),
                                    ""
                                )
                            ),
                            e
                        )
                    )
                });
            }
        }
    }
}

/// downloads tarball file from package
pub async fn download_tarball(app: &App, package: &VoltPackage, secure: bool) -> Result<()> {
    let package_instance = package.clone();

    // @types/eslint
    if package_instance.name.starts_with('@') && package_instance.name.contains("/") {
        let package_directory_location = app
            .volt_dir
            .join(&package.name.split("/").collect::<Vec<&str>>()[0]);

        if !Path::new(&package_directory_location).exists() {
            create_dir_all(&package_directory_location).await.unwrap();
        }
    }

    // location of extracted package
    let loc = app.volt_dir.join(&package.name);

    // if package is not already installed
    if !Path::new(&loc).exists() {
        // Url to download tarball code files from
        let mut url = package_instance.tarball;
        let registries = vec!["npmjs.org", "yarnpkg.com"];
        let random_registry = registries.choose(&mut rand::thread_rng()).unwrap();

        url = url.replace("npmjs.org", random_registry);

        if !secure {
            url = url.replace("https", "http")
        }

        // Get Tarball File
        let res = reqwest::get(url).await.unwrap();

        let bytes: bytes::Bytes = res.bytes().await.unwrap();

        // Verify If Bytes == Sha1
        if package.sha1 == App::calc_hash(&bytes).unwrap() {
            // println!("{} => {}", url.clone(), loc);
            // Create node_modules
            create_dir_all(&app.node_modules_dir).await?;

            // Delete package from node_modules
            let node_modules_dep_path = app.node_modules_dir.join(&package.name);

            if node_modules_dep_path.exists() {
                remove_dir_all(&node_modules_dep_path)?;
            }

            // Directory to extract tarball to
            let mut extract_directory = PathBuf::from(&app.volt_dir);

            // @types/eslint
            if package.clone().name.starts_with('@') && package.clone().name.contains("/") {
                if cfg!(target_os = "windows") {
                    let name = package.clone().name.replace(r"/", r"\");

                    let split = name.split(r"\").collect::<Vec<&str>>();

                    // C:\Users\xtrem\.volt\@types
                    extract_directory = extract_directory.join(split[0]);
                } else {
                    let name = package.clone().name;
                    let split = name.split('/').collect::<Vec<&str>>();

                    // ~/.volt/@types
                    extract_directory = extract_directory.join(split[0]);
                }
            }

            // if !package.clone().name.starts_with("@") && !package.clone().name.contains("/") {
            extract_directory = extract_directory.join(package.clone().name);
            // } else {
            //     let name = package.clone().name;
            //     let split = name.split("/").collect::<Vec<&str>>();
            //     extract_directory = extract_directory.join(split[1]);
            // }
            // println!("{}", extract_directory.display());

            // Initialize tarfile decoder while directly passing in bytes
            let gz_decoder = GzDecoder::new(&*bytes);

            let mut archive = Archive::new(gz_decoder);

            // Extract the data into extract_directory
            archive
                .unpack(&extract_directory)
                .context("Unable to unpack dependency")?;

            drop(bytes);

            if cfg!(target_os = "windows") {
                if Path::new(format!(r"{}\package", &extract_directory.to_str().unwrap()).as_str())
                    .exists()
                {
                    std::fs::rename(
                        format!(r"{}\package", &extract_directory.to_str().unwrap()),
                        format!(
                            r"{}\{}",
                            &extract_directory.to_str().unwrap(),
                            package.clone().version
                        ),
                    )
                    .context("failed to rename dependency folder")
                    .unwrap_and_handle_error();
                } else {
                    if Path::new(
                        format!(r"{}/package", &extract_directory.to_str().unwrap()).as_str(),
                    )
                    .exists()
                    {
                        std::fs::rename(
                            format!(r"{}/package", &extract_directory.to_str().unwrap()),
                            format!(
                                r"{}/{}",
                                &extract_directory.to_str().unwrap(),
                                package.clone().version
                            ),
                        )
                        .context("failed to rename dependency folder")
                        .unwrap_and_handle_error();
                    }
                }
            } else {
                if Path::new(format!(r"{}/package", &extract_directory.to_str().unwrap()).as_str())
                    .exists()
                {
                    std::fs::rename(
                        format!(r"{}/package", &extract_directory.to_str().unwrap()),
                        format!(
                            r"{}/{}",
                            &extract_directory.to_str().unwrap(),
                            package.clone().version
                        ),
                    )
                    .context("failed to rename dependency folder")
                    .unwrap_and_handle_error();
                }
            }
            if let Some(parent) = node_modules_dep_path.parent() {
                if !parent.exists() {
                    create_dir_all(&parent).await?;
                }
            }
        } else {
            return Err(anyhow::Error::msg("failed to verify checksum"));
        }
    }

    Ok(())
}

pub async fn download_tarball_create(
    _app: &App,
    package: &Package,
    name: &str,
) -> Result<String, Error> {
    let file_name = format!("{}-{}.tgz", name, package.dist_tags.get("latest").unwrap());
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
    let package_version = package
        .versions
        .get(package.dist_tags.get("latest").unwrap())
        .unwrap();

    let bytes = std::fs::read(path_str.clone()).unwrap();

    // Corrupt tar files may cause issues
    if let Ok(hash) = App::calc_hash(&bytes::Bytes::from(bytes)) {
        // File exists, make sure it's not corrupted
        if hash
            == package
                .versions
                .get(package.dist_tags.get("latest").unwrap())
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
pub fn get_git_config(key: &str) -> std::io::Result<Option<String>> {
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
/// Enable ansi support and colors
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

#[cfg(windows)]
pub fn generate_script(app: &Arc<App>, package: &VoltPackage) {
    // Create node_modules/scripts if it doesn't exist
    if !Path::new("node_modules/scripts").exists() {
        std::fs::create_dir_all("node_modules/scripts").unwrap();
    }

    // If the package has binary scripts, create them
    if package.bin.is_some() {
        let bin = package.bin.as_ref().unwrap();

        let k = bin.keys().next().unwrap();
        let v = bin.values().next().unwrap();

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
        .replace(r"%~dp0\..", format!("{}", app.volt_dir.display()).as_str());

        let mut f = File::create(format!(r"node_modules/scripts/{}.cmd", k)).unwrap();
        f.write_all(command.as_bytes()).unwrap();
    }
}

#[cfg(unix)]
pub fn generate_script(app: &Arc<App>, package: &VoltPackage) {
    // Create node_modules/scripts if it doesn't exist
    if !Path::new("node_modules/scripts").exists() {
        std::fs::create_dir_all("node_modules/scripts").unwrap();
    }

    // If the package has binary scripts, create them
    if package.bin.is_some() {
        let bin = package.bin.as_ref().unwrap();

        let k = bin.keys().next().unwrap();
        let v = bin.values().next().unwrap();

        let command = format!(
            r#"
node  "{}/.volt/{}/{}" %*
"#,
            app.volt_dir.to_string_lossy(),
            k,
            v,
        );
        // .replace(r"%~dp0\..", format!("{}", app.volt_dir.display()).as_str());
        let p = format!(r"node_modules/scripts/{}.sh", k);
        let mut f = File::create(p.clone()).unwrap();
        std::process::Command::new("chmod")
            .args(&["+x", &p])
            .spawn()
            .unwrap();
        f.write_all(command.as_bytes()).unwrap();
    }
}
// Unix functions
#[cfg(unix)]
pub fn enable_ansi_support() -> Result<(), u32> {
    Ok(())
}

pub fn check_peer_dependency(_package_name: &str) -> bool {
    false
}

pub async fn install_extract_package(app: &Arc<App>, package: &VoltPackage) -> Result<()> {
    let pb = ProgressBar::new(0);
    let text = format!("{}", "Installing Packages".bright_cyan());

    pb.set_style(
        ProgressStyle::default_spinner()
            .template(("{spinner:.green}".to_string() + format!(" {}", text).as_str()).as_str())
            .tick_strings(&["┤", "┘", "┴", "└", "├", "┌", "┬", "┐"]),
    );

    if download_tarball(&app, &package, false).await.is_err() {
        download_tarball(&app, &package, true)
            .await
            .unwrap_or_else(|_| {
                println!("Failed");
                std::process::exit(1);
            });
    }

    generate_script(&app, package);

    let directory = &app.volt_dir.join(package.name.clone());

    let path = Path::new(directory.as_os_str());

    // hardlink_files(app.to_owned(), &path).await;

    Ok(())
}

// Credit: https://gist.github.com/ZaphodElevated/059faa3c0c605f03369d0f84b9c8cfb9
async fn threaded_download(threads: u64, url: &String, output: &str) {
    let mut handles = vec![];

    // Create response from url
    let res = reqwest::get(url.to_string()).await.unwrap();

    // Get the total bytes of the response
    let total_length = res.content_length().unwrap();

    // Create buffer for bytes
    let mut buffer: Vec<u8> = vec![];

    for index in 0..threads {
        let mut buf: Vec<u8> = vec![];
        let url = url.clone();

        let (start, end) = get_splits(index + 1, total_length, threads);

        handles.push(tokio::spawn(async move {
            let client = reqwest::Client::new();

            let mut response = client
                .get(url)
                .header("range", format!("bytes={}-{}", start, end))
                .send()
                .await
                .unwrap();

            while let Some(chunk) = response.chunk().await.unwrap() {
                let _ = std::io::copy(&mut &*chunk, &mut buf);
            }

            buf
        }))
    }

    // Join all handles
    let result = futures::future::join_all(handles).await;
    for res in result {
        buffer.append(&mut res.unwrap().clone());
    }

    let mut file = File::create(output.clone()).unwrap();

    let _ = file.write_all(&buffer);
}

fn get_splits(i: u64, total_length: u64, threads: u64) -> (u64, u64) {
    let mut start = ((total_length / threads) * (i - 1)) + 1;
    let mut end = (total_length / threads) * i;

    if i == 1 {
        start = 0;
    }

    if i == threads {
        end = total_length;
    }

    (start, end)
}
