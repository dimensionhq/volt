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

pub mod app;
pub mod constants;
pub mod errors;
pub mod helper;
pub mod npm;
pub mod package;
pub mod scripts;
pub mod voltapi;

use crate::commands::add::PackageInfo;
use crate::core::utils::constants::MAX_RETRIES;
use crate::core::utils::voltapi::JSONVoltResponse;
use crate::core::utils::voltapi::{VoltPackage, VoltResponse};

use app::App;
use colored::Colorize;
use errors::VoltError;
use flate2::read::GzDecoder;
use fs_extra::dir::CopyOptions;
use futures_util::{stream::FuturesUnordered, StreamExt};
use git_config::{file::GitConfig, parser::Parser};
use indicatif::ProgressBar;
use isahc::AsyncReadResponseExt;
use miette::Result;
use reqwest::StatusCode;
use ssri::{Algorithm, Integrity};
use tar::Archive;
use tokio::fs::create_dir_all;

use std::{
    collections::HashMap,
    convert::TryFrom,
    ffi::OsStr,
    fs::{read_to_string, File},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    sync::Arc,
};

pub struct State {}

//pub struct State {
//pub http_client: Client,
//}

/// convert a JSONVoltResponse -> VoltResponse
pub fn convert(version: String, deserialized: JSONVoltResponse) -> Result<VoltResponse> {
    // initialize a hashmap to store the converted versions
    let mut converted_versions: HashMap<String, VoltPackage> = HashMap::new();

    // iterate through all listed dependencies of the latest version of the response
    for version in deserialized.versions {
        // access data in the hashmap, not name@version
        let data = version.1;

        // @codemirror/state -> state
        let split = version
            .0
            .split('@')
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>();

        let mut package_name = String::new();

        if split.len() == 2 {
            package_name = split[0].to_string();
            if package_name.contains('/') {
                package_name = format!("@{}", package_name);
            }
        }

        // @codemirror/state@1.2.3 -> 1.2.3
        let package_version = version.0.split('@').last().unwrap();

        let integrity: Integrity =
            data.integrity
                .clone()
                .parse()
                .map_err(|_| VoltError::HashParseError {
                    hash: data.integrity.clone(),
                })?;

        let algo = integrity.pick_algorithm();

        let mut hash = integrity
            .hashes
            .into_iter()
            .find(|h| h.algorithm == algo)
            .map(|h| Integrity { hashes: vec![h] })
            .map(|i| i.to_hex().1)
            .ok_or(VoltError::IntegrityConversionError)?;

        match algo {
            Algorithm::Sha1 => {
                hash = format!("sha1-{}", hash);
            }
            Algorithm::Sha512 => {
                hash = format!("sha512-{}", hash);
            }
            _ => {}
        }

        converted_versions.insert(
            version.0.to_string(), // name@version
            VoltPackage {
                name: package_name.to_string(),
                version: package_version.to_string(),
                tarball: data.tarball.clone(),
                bin: data.bin.clone(),
                integrity: hash,
                peer_dependencies: data.peer_dependencies.clone(),
                dependencies: data.dependencies.clone(),
            },
        );
    }

    // create a final hashmap

    Ok(VoltResponse {
        version,
        versions: converted_versions,
    })
}

pub async fn get_volt_response_multi(
    versions: &[(PackageInfo, String, VoltPackage, bool)],
    pb: &ProgressBar,
) -> Vec<Result<VoltResponse>> {
    versions
        .iter()
        .map(|(package_info, hash, package, no_deps)| {
            get_volt_response(package_info, hash, package.to_owned(), *no_deps)
        })
        .collect::<FuturesUnordered<_>>()
        .inspect(|_| pb.inc(1))
        .collect::<Vec<Result<VoltResponse>>>()
        .await
}

// Get response from volt CDN
pub async fn get_volt_response(
    package_info: &PackageInfo,
    hash: &str,
    package: VoltPackage,
    zero_deps: bool,
) -> Result<VoltResponse> {
    // number of retries
    let mut retries = 0;

    // only 1 package, zero dependencies
    if zero_deps {
        let mut versions: HashMap<String, VoltPackage> = HashMap::new();

        versions.insert(
            format!("{}@{}", package.clone().version, package.clone().name),
            package.clone(),
        );

        return Ok(VoltResponse {
            version: package.version,
            versions,
        });
    }

    // loop until MAX_RETRIES reached.
    loop {
        // get a response
        let mut response = isahc::get_async(format!("http://registry.voltpkg.com/{}.json", hash))
            .await
            .map_err(VoltError::NetworkError)?;

        // check the status of the response
        match response.status() {
            // 200 (OK)
            StatusCode::OK => {
                let deserialized = response
                    .json()
                    .await
                    .map_err(|_| VoltError::DeserializeError)?;

                let converted = convert(package.version, deserialized)?;

                return Ok(converted);
            }
            // 429 (TOO_MANY_REQUESTS)
            StatusCode::TOO_MANY_REQUESTS => {
                return Err(VoltError::TooManyRequests {
                    url: format!("http://registry.voltpkg.com/{}", package_info.name),
                    package_name: package_info.name.clone(),
                }
                .into());
            }
            // 400 (BAD_REQUEST)
            StatusCode::BAD_REQUEST => {
                return Err(VoltError::BadRequest {
                    url: format!("http://registry.voltpkg.com/{}", package_info.name),
                    package_name: package_info.name.clone(),
                }
                .into());
            }
            // 404 (NOT_FOUND)
            StatusCode::NOT_FOUND if retries == MAX_RETRIES => {
                return Err(VoltError::PackageNotFound {
                    url: format!("http://registry.voltpkg.com/{}", package_info.name),
                    package_name: package_info.name.clone(),
                }
                .into());
            }
            // Other Errors
            _ => {
                if retries == MAX_RETRIES {
                    return Err(VoltError::NetworkUnknownError {
                        url: format!("http://registry.voltpkg.com/{}", package_info.name),
                        package_name: package_info.name.clone(),
                        code: response.status().as_str().to_string(),
                    }
                    .into());
                }
            }
        }

        retries += 1;
    }
}

// #[cfg(windows)]
// pub async fn hardlink_files(app: Arc<App>, src: PathBuf) {
//     for entry in WalkDir::new(src) {
//         let entry = entry.unwrap();

//         if !entry.path().is_dir() {
//             // index.js
//             let entry = entry.path();

//             let file_name = entry.file_name().unwrap().to_str().unwrap();

//             // lib/index.js
//             let path = format!("{}", &entry.display())
//                 .replace(r"\", '/')
//                 .replace(&app.volt_dir.display().to_string(), "");

//             // node_modules/lib
//             create_dir_all(format!(
//                 "node_modules/{}",
//                 &path
//                     .replace(
//                         format!("{}", &app.volt_dir.display())
//                             .replace(r"\", '/')
//                             .as_str(),
//                         ""
//                     )
//                     .trim_end_matches(file_name)
//             ))
//             .await
//             .unwrap();

//             // ~/.volt/package/lib/index.js -> node_modules/package/lib/index.js
//             if !Path::new(&format!(
//                 "node_modules{}",
//                 &path.replace(
//                     format!("{}", &app.volt_dir.display())
//                         .replace(r"\", '/')
//                         .as_str(),
//                     ""
//                 )
//             ))
//             .exists()
//             {
//                 hard_link(
//                     format!("{}", &path),
//                     format!(
//                         "node_modules{}",
//                         &path.replace(
//                             format!("{}", &app.volt_dir.display())
//                                 .replace(r"\", '/')
//                                 .as_str(),
//                             ""
//                         )
//                     ),
//                 )
//                 .await
//                 .unwrap_or_else(|_| {
//                     0;
//                 });
//             }
//         }
//     }
// }

// #[cfg(unix)]
// pub async fn hardlink_files(app: Arc<App>, src: PathBuf) {
//     let mut src = src;
//     let volt_directory = format!("{}", app.volt_dir.display());

//     if !cfg!(target_os = "windows") {
//         src = src.replace(r"\", '/');
//     }

//     for entry in WalkDir::new(src) {
//         let entry = entry.unwrap();

//         if !entry.path().is_dir() {
//             // index.js
//             let file_name = &entry.path().file_name().unwrap().to_str().unwrap();

//             // lib/index.js
//             let path = format!("{}", &entry.path().display())
//                 .replace(r"\", '/')
//                 .replace(&volt_directory, "");

//             // node_modules/lib
//             create_dir_all(format!(
//                 "node_modules/{}",
//                 &path
//                     .replace(
//                         format!("{}", &app.volt_dir.display())
//                             .replace(r"\", '/')
//                             .as_str(),
//                         ""
//                     )
//                     .trim_end_matches(file_name)
//             ))
//             .await
//             .unwrap();

//             // ~/.volt/package/lib/index.js -> node_modules/package/lib/index.js
//             if !Path::new(&format!(
//                 "node_modules{}",
//                 &path.replace(
//                     format!("{}", &app.volt_dir.display())
//                         .replace(r"\", '/')
//                         .as_str(),
//                     ""
//                 )
//             ))
//             .exists()
//             {
//                 hard_link(
//                     format!("{}/.volt/{}", std::env::var("HOME").unwrap(), path),
//                     format!(
//                         "{}/node_modules{}",
//                         std::env::current_dir().unwrap().to_string_lossy(),
//                         &path.replace(
//                             format!("{}", &app.volt_dir.display())
//                                 .replace(r"\", '/')
//                                 .as_str(),
//                             ""
//                         )
//                     ),
//                 )
//                 .await
//                 .unwrap_or_else(|e| {
//                     panic!(
//                         "{:#?}",
//                         (
//                             format!("{}", &path),
//                             format!(
//                                 "node_modules{}",
//                                 &path.replace(
//                                     format!("{}", &app.volt_dir.display())
//                                         .replace(r"\", '/')
//                                         .as_str(),
//                                     ""
//                                 )
//                             ),
//                             e
//                         )
//                     )
//                 });
//             }
//         }
//     }
// }

/// downloads and extracts tarball file from package
pub async fn download_tarball(app: &App, package: VoltPackage, _state: State) -> Result<()> {
    let package_instance = package.clone();

    let package_name = package.name.clone();
    let package_version = package.version.clone();

    // @types/eslint
    if package_name.starts_with('@') && package_name.contains('/') {
        let package_directory_location = app
            .volt_dir
            .join(&package.name.split('/').collect::<Vec<&str>>()[0]);

        if !Path::new(&package_directory_location).exists() {
            create_dir_all(&package_directory_location)
                .await
                .map_err(VoltError::CreateDirError)?;
        }
    }

    // location of extracted package
    let loc = app
        .volt_dir
        .join(format!("{}-{}", &package_name, &package_version));

    let client = reqwest::ClientBuilder::new()
        .use_rustls_tls()
        .build()
        .unwrap();

    // if package is not already installed
    if !Path::new(&loc).exists() {
        // Tarball bytes response
        let bytes: bytes::Bytes = client
            .get(package_instance.tarball)
            .send()
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap();

        let algorithm;

        // there are only 2 supported algorithms
        // sha1 and sha512
        // so we can be sure that if it doesn't start with sha1, it's going to have to be sha512
        if package.integrity.starts_with("sha1") {
            algorithm = Algorithm::Sha1;
        } else {
            algorithm = Algorithm::Sha512;
        }

        // Verify If Bytes == (Sha512 | Sha1) of Tarball
        if package.integrity == App::calc_hash(&bytes, algorithm).unwrap() {
            // Create node_modules
            create_dir_all(&app.node_modules_dir).await.unwrap();

            // Directory to extract tarball to
            let mut extract_directory = PathBuf::from(&app.volt_dir);

            // @types/eslint
            if package.clone().name.starts_with('@') && package.clone().name.contains('/') {
                if cfg!(target_os = "windows") {
                    let name = package.clone().name.replace('/', r"\");

                    let split = name.split('\\').collect::<Vec<&str>>();

                    // C:\Users\xtrem\.volt\@types
                    extract_directory = extract_directory.join(split[0]);
                } else {
                    let name = package.clone().name;
                    let split = name.split('/').collect::<Vec<&str>>();

                    // ~/.volt/@types
                    extract_directory = extract_directory.join(split[0]);
                }
            }

            extract_directory =
                extract_directory.join(format!("{}-{}", &package_name, &package_version,));

            // Initialize tarfile decoder while directly passing in bytes

            let bytes = Arc::new(bytes);

            let bytes_ref = bytes.clone();

            let node_modules_dep_path_instance = app.node_modules_dir.clone();

            futures::try_join!(
                tokio::task::spawn_blocking(move || {
                    // Extract the data into extract_directory

                    let node_gz_decoder = GzDecoder::new(&**bytes_ref);

                    let mut node_archive = Archive::new(node_gz_decoder);

                    for entry in node_archive.entries().unwrap() {
                        let mut entry = entry.unwrap();
                        let path = entry.path().unwrap();
                        let mut new_path = PathBuf::new();

                        for component in path.components() {
                            if component.as_os_str() == "package" {
                                new_path.push(Component::Normal(OsStr::new(&package.name)));
                            } else {
                                new_path.push(component)
                            }
                        }

                        match entry
                            .unpack(node_modules_dep_path_instance.to_path_buf().join(&new_path))
                        {
                            Ok(_v) => {}
                            Err(_err) => {}
                        }
                    }
                }),
                tokio::task::spawn_blocking(move || {
                    let gz_decoder = GzDecoder::new(&**bytes);

                    let mut archive = Archive::new(gz_decoder);

                    for entry in archive.entries().unwrap() {
                        let mut entry = entry.unwrap();
                        // let path = entry.path().unwrap();
                        // let mut new_path = PathBuf::new();

                        // for component in path.components() {
                        //     if component.as_os_str() == "package" {
                        //         new_path.push(Component::Normal(OsStr::new(&pkg_name_instance)));
                        //     } else {
                        //         new_path.push(component)
                        //     }
                        // }

                        let mut buffer = vec![];

                        entry.read_to_end(&mut buffer).unwrap();

                        let sri = cacache::write_sync(
                            extract_directory.clone(),
                            format!("pkg::{}::{}", &package_name, &package_version),
                            &buffer,
                        )
                        .unwrap();
                    }
                })
            )
            .unwrap();
        } else {
            return Err(VoltError::ChecksumVerificationError.into());
        }
    } else {
        // package is already downloaded and extracted to the ~/.volt folder.
        let node_modules_path = Path::new("node_modules/").join(package_instance.name);

        fs_extra::dir::copy(loc, node_modules_path, &CopyOptions::new()).unwrap();
    }

    Ok(())
}

/// Gets a config key from git using the git cli.
/// Uses `gitoxide` to read from your git configuration.
pub fn get_git_config(app: &App, key: &str) -> Option<String> {
    match key {
        "user.name" => {
            let config_path = app.home_dir.join(".gitconfig");

            if !config_path.exists() {
                None
            } else {
                let data = read_to_string(config_path).ok()?;

                let config = GitConfig::from(Parser::try_from(data.as_str()).ok()?);
                let value = config.get_raw_value("user", None, "name").ok()?;

                Some(String::from_utf8_lossy(&value).to_owned().to_string())
            }
        }
        "user.email" => {
            let config_path = app.home_dir.join(".gitconfig");

            if !config_path.exists() {
                None
            } else {
                let data = read_to_string(config_path).ok()?;

                let config = GitConfig::from(Parser::try_from(data.as_str()).ok()?);
                let value = config.get_raw_value("user", None, "email").ok()?;

                Some(String::from_utf8_lossy(&value).to_owned().to_string())
            }
        }
        "repository.url" => {
            let remote_config_path = app.current_dir.join(".git").join("config");

            if !remote_config_path.exists() {
                let data = read_to_string(remote_config_path).ok()?;

                let config = GitConfig::from(Parser::try_from(data.as_str()).ok()?);
                let value = config.get_raw_value("remote", Some("origin"), "url").ok()?;

                Some(String::from_utf8_lossy(&value).to_owned().to_string())
            } else {
                None
            }
        }
        _ => None,
    }
}

// Windows Function
/// Enable ansi support and colors
#[cfg(windows)]
pub fn enable_ansi_support() -> Result<(), u32> {
    // ref: https://docs.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences#EXAMPLE_OF_ENABLING_VIRTUAL_TERMINAL_PROCESSING @@ https://archive.is/L7wRJ#76%

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
/// Generates the binary and other required scripts for the package
pub fn generate_script(app: &Arc<App>, package: &VoltPackage) {
    // Create node_modules/scripts if it doesn't exist
    if !Path::new("node_modules/.bin").exists() {
        // Create the binary directory
        std::fs::create_dir_all("node_modules/.bin").unwrap();
    }

    // Create binary scripts for the package if they exist.
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

        let mut f = File::create(format!(r"node_modules/.bin/{}.cmd", k)).unwrap();
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

/// package all steps for installation into 1 convenient function.
pub async fn install_package(app: &Arc<App>, package: &VoltPackage, state: State) -> Result<()> {
    if download_tarball(app, package.clone(), state).await.is_err() {}

    // generate the package's script
    generate_script(app, package);

    // let directory = &app
    //     .volt_dir
    //     .join(package.version.clone())
    //     .join(package.name.clone());

    // let path = Path::new(directory.as_os_str());

    // hardlink_files(app.to_owned(), (&path).to_path_buf()).await;

    Ok(())
}

pub async fn fetch_dep_tree(
    data: &[(PackageInfo, String, VoltPackage, bool)],
    progress_bar: &ProgressBar,
) -> Result<Vec<VoltResponse>> {
    if data.len() > 1 {
        Ok(get_volt_response_multi(data, progress_bar)
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?)
    } else {
        Ok(vec![
            get_volt_response(&data[0].0, &data[0].1, data[0].2.clone(), data[0].3).await?,
        ])
    }
}

pub fn print_elapsed(length: usize, elapsed: f32) {
    if length == 1 {
        if elapsed < 0.001 {
            println!(
                "{}: resolved 1 dependency in {:.5}s.",
                "success".bright_green(),
                elapsed
            );
        } else {
            println!(
                "{}: resolved 1 dependency in {:.2}s.",
                "success".bright_green(),
                elapsed
            );
        }
    } else if elapsed < 0.001 {
        println!(
            "{}: resolved {} dependencies in {:.4}s.",
            "success".bright_green(),
            length,
            elapsed
        );
    } else {
        println!(
            "{}: resolved {} dependencies in {:.2}s.",
            "success".bright_green(),
            length,
            elapsed
        );
    }
}
