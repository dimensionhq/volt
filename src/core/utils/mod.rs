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
pub mod package;
pub mod scripts;
pub mod voltapi;

use crate::core::{
    utils::constants::MAX_RETRIES,
    utils::voltapi::{VoltPackage, VoltResponse},
};

use app::App;
use colored::Colorize;
use errors::VoltError;
// use flate2::read::GzDecoder;
use futures_util::{stream::FuturesUnordered, StreamExt};
use git_config::{file::GitConfig, parser::Parser};
use indicatif::ProgressBar;
use isahc::AsyncReadResponseExt;
use miette::Result;
use package_spec::PackageSpec;
use reqwest::{Client, StatusCode};
use speedy::Readable;
use ssri::{Algorithm, Integrity};
use tar::Archive;

use std::{
    collections::HashMap,
    convert::TryFrom,
    ffi::OsStr,
    fs::{read_to_string, File},
    io::{Cursor, Read, Write},
    path::{Component, Path, PathBuf},
    sync::Arc,
};

pub struct State {
    pub http_client: Client,
}

//pub struct State {
//}
pub async fn get_volt_response_multi(
    packages: &[PackageSpec],
    progress_bar: &ProgressBar,
) -> Vec<Result<VoltResponse>> {
    packages
        .iter()
        .map(|spec| {
            if let PackageSpec::Npm {
                name,
                requested,
                scope,
            } = spec
            {
                let mut version: String = "latest".to_string();

                if requested.is_some() {
                    version = requested.as_ref().unwrap().to_string();
                };

                progress_bar.set_message(format!("{}@{}", name, version.truecolor(125, 125, 125)));
            }

            get_volt_response(spec)
        })
        .collect::<FuturesUnordered<_>>()
        .collect::<Vec<Result<VoltResponse>>>()
        .await
}

// Get response from volt CDN
pub async fn get_volt_response(package_spec: &PackageSpec) -> Result<VoltResponse> {
    // number of retries
    let mut retries = 0;

    // we know that PackageSpec is of type npm (we filtered the non-npm ones out)

    if let PackageSpec::Npm {
        name,
        requested,
        scope,
    } = package_spec
    {
        // loop until MAX_RETRIES reached.
        loop {
            // get a response
            let mut response =
                isahc::get_async(format!("http://registry.voltpkg.com/{}.sp", &package_spec))
                    .await
                    .map_err(VoltError::NetworkError)?;

            // check the status of the response
            match response.status() {
                // 200 (OK)
                StatusCode::OK => {
                    let response: VoltResponse =
                        VoltResponse::read_from_buffer(&response.bytes().await.unwrap()).unwrap();

                    return Ok(response);
                }
                // 429 (TOO_MANY_REQUESTS)
                StatusCode::TOO_MANY_REQUESTS => {
                    return Err(VoltError::TooManyRequests {
                        url: format!("http://registry.voltpkg.com/{}.sp", &package_spec),
                    }
                    .into());
                }
                // 400 (BAD_REQUEST)
                StatusCode::BAD_REQUEST => {
                    return Err(VoltError::BadRequest {
                        url: format!("http://registry.voltpkg.com/{}.sp", &package_spec),
                    }
                    .into());
                }
                // 404 (NOT_FOUND)
                StatusCode::NOT_FOUND if retries == MAX_RETRIES => {
                    return Err(VoltError::PackageNotFound {
                        url: format!("http://registry.voltpkg.com/{}.sp", &package_spec),
                        package_name: package_spec.to_string(),
                    }
                    .into());
                }
                // Other Errors
                _ => {
                    if retries == MAX_RETRIES {
                        return Err(VoltError::NetworkUnknownError {
                            url: format!("http://registry.voltpkg.com/{}.sp", name),
                            package_name: package_spec.to_string(),
                            code: response.status().as_str().to_string(),
                        }
                        .into());
                    }
                }
            }

            retries += 1;
        }
    } else {
        panic!("Volt does not support non-npm package specifications yet.");
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

pub fn decompress_tarball(gz_data: &[u8]) -> Vec<u8> {
    // gzip RFC1952: a valid gzip file has an ISIZE field in the
    // footer, which is a little-endian u32 number representing the
    // decompressed size. This is ideal for libdeflate, which needs
    // preallocating the decompressed buffer.
    let isize = {
        let isize_start = gz_data.len() - 4;
        let isize_bytes: [u8; 4] = gz_data[isize_start..]
            .try_into()
            .expect("we know the end has 4 bytes");
        u32::from_le_bytes(isize_bytes) as usize
    };

    let mut decompressor = libdeflater::Decompressor::new();
    let mut outbuf = Vec::with_capacity(isize);
    outbuf.resize(isize, 0);
    decompressor.gzip_decompress(gz_data, &mut outbuf).unwrap();

    outbuf
}

/// downloads and extracts tarball file from package
pub async fn download_tarball(app: &App, package: VoltPackage, state: State) -> Result<()> {
    let package_instance = package.clone();

    let package_name = package.name.clone();
    let package_version = package.version.clone();

    let global_cas_directory = PathBuf::from(&app.volt_dir);

    let existing_check = cacache::read_sync(
        &global_cas_directory,
        format!(
            "pkg::{}::{}::{}",
            &package_name, &package_version, package.integrity
        ),
    );

    // if package is not already installed
    if existing_check.is_err() {
        // Tarball bytes response
        let bytes: bytes::Bytes = state
            .http_client
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
            // decompress gzipped tarball
            let decompressed_bytes = decompress_tarball(&bytes);

            // Generate the tarball archive given the decompressed bytes
            let mut node_archive = Archive::new(Cursor::new(decompressed_bytes));

            // extract to both the global store + node_modules (in the case of them using the pnpm linking algorithm)
            let mut cas_file_map: HashMap<String, Integrity> = HashMap::new();

            for entry in node_archive.entries().unwrap() {
                let mut entry = entry.unwrap();
                let mut buffer = vec![];

                entry.read_to_end(&mut buffer).unwrap();

                let path = entry.path().unwrap().to_str().unwrap().to_string();

                let cleaned_path = if let Some(i) = path.char_indices().position(|(_, c)| c == '/')
                {
                    &path[i + 1..]
                } else {
                    &path[..]
                };

                let write_path = app
                    .node_modules_dir
                    .join("node_modules/.volt")
                    .join(format!(
                        "{}@{}",
                        package_instance.name, package_instance.version
                    ))
                    .join(cleaned_path);

                let parent = write_path.parent().unwrap();

                if parent
                    != app
                        .node_modules_dir
                        .join("node_modules/.volt")
                        .join(format!(
                            "{}@{}",
                            package_instance.name, package_instance.version
                        ))
                {
                    std::fs::create_dir_all(&write_path.parent().unwrap()).unwrap();
                }

                let sri = cacache::write_hash_sync(global_cas_directory.clone(), &buffer).unwrap();

                cas_file_map.insert(entry.path().unwrap().to_str().unwrap().to_string(), sri);
            }

            cacache::write_sync(
                global_cas_directory,
                format!(
                    "pkg::{}::{}::{}",
                    &package_name, &package_version, package.integrity
                ),
                serde_json::to_string(&cas_file_map).unwrap(),
            )
            .unwrap();
        } else {
            println!(
                "{} vs {}",
                package.integrity,
                App::calc_hash(&bytes, algorithm).unwrap()
            );
            return Err(VoltError::ChecksumVerificationError.into());
        }
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
    // // Create node_modules/scripts if it doesn't exist
    // if !Path::new("node_modules/.bin").exists() {
    //     // Create the binary directory
    //     std::fs::create_dir_all("node_modules/.bin").unwrap();
    // }

    // // Create binary scripts for the package if they exist.
    // if package.bin.is_some() {
    //     let bin = package.bin.as_ref().unwrap();

    //     let k = bin.keys().next().unwrap();
    //     let v = bin.values().next().unwrap();

    //     let command = format!(
    //         r#"
    //         @IF EXIST "%~dp0\node.exe" (
    //             "%~dp0\node.exe"  "%~dp0\..\{}\{}" %*
    //             ) ELSE (
    //                 @SETLOCAL
    //                 @SET PATHEXT=%PATHEXT:;.JS;=;%
    //                 node  "%~dp0\..\{}\{}" %*
    //                 )"#,
    //         k, v, k, v
    //     )
    //     .replace(r"%~dp0\..", format!("{}", app.volt_dir.display()).as_str());

    //     let mut f = File::create(format!(r"node_modules/.bin/{}.cmd", k)).unwrap();
    //     f.write_all(command.as_bytes()).unwrap();
    // }
}

#[cfg(unix)]
pub fn generate_script(app: &Arc<App>, package: &VoltPackage) {
    // Create node_modules/scripts if it doesn't exist
    // if !Path::new("node_modules/scripts").exists() {
    //     std::fs::create_dir_all("node_modules/scripts").unwrap();
    // }

    // // If the package has binary scripts, create them
    // if package.bin.is_some() {
    //     let bin = package.bin.as_ref().unwrap();

    //     let k = bin.keys().next().unwrap();
    //     let v = bin.values().next().unwrap();

    //     let command = format!(
    //         r#"
    //         node  "{}/.volt/{}/{}" %*
    //         "#,
    //         app.volt_dir.to_string_lossy(),
    //         k,
    //         v,
    //     );
    //     // .replace(r"%~dp0\..", format!("{}", app.volt_dir.display()).as_str());
    //     let p = format!(r"node_modules/scripts/{}.sh", k);
    //     let mut f = File::create(p.clone()).unwrap();
    //     std::process::Command::new("chmod")
    //         .args(&["+x", &p])
    //         .spawn()
    //         .unwrap();
    //     f.write_all(command.as_bytes()).unwrap();
    // }
}

// Unix functions
#[cfg(unix)]
pub fn enable_ansi_support() -> Result<(), u32> {
    Ok(())
}

pub fn check_peer_dependency(_package_name: &str) -> bool {
    false
}

/// Install process for any package.
pub async fn install_package(app: Arc<App>, package: &VoltPackage, state: State) -> Result<()> {
    if download_tarball(&app, package.clone(), state)
        .await
        .is_err()
    {}

    // generate the package's script
    // generate_script(app, package);

    // let directory = &app
    //     .volt_dir
    //     .join(package.version.clone())
    //     .join(package.name.clone());

    // let path = Path::new(directory.as_os_str());

    // hardlink_files(app.to_owned(), (&path).to_path_buf()).await;

    Ok(())
}

pub async fn fetch_dep_tree(
    data: &[PackageSpec],
    progress_bar: &ProgressBar,
) -> Result<Vec<VoltResponse>> {
    if data.len() > 1 {
        Ok(get_volt_response_multi(data, progress_bar)
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?)
    } else {
        if let PackageSpec::Npm {
            name,
            requested,
            scope,
        } = &data[0]
        {
            let mut version: String = "latest".to_string();

            if requested.is_some() {
                version = requested.as_ref().unwrap().to_string();
            };

            progress_bar.set_message(format!("{}@{}", name, version.truecolor(125, 125, 125)));
        }

        Ok(vec![get_volt_response(&data[0]).await?])
    }
}

// pub fn print_elapsed(length: usize, elapsed: f32) {
// if length == 1 {
//     if elapsed < 0.001 {
//         println!(
//             "{}: resolved 1 dependency in {:.5}s.",
//             "success".bright_green(),
//             elapsed
//         );
//     } else {
//         println!(
//             "{}: resolved 1 dependency in {:.2}s.",
//             "success".bright_green(),
//             elapsed
//         );
//     }
// } else if elapsed < 0.001 {
//     println!(
//         "{}: resolved {} dependencies in {:.4}s.",
//         "success".bright_green(),
//         length,
//         elapsed
//     );
// } else {
//     println!(
//         "{}: resolved {} dependencies in {:.2}s.",
//         "success".bright_green(),
//         length,
//         elapsed
//     );
// }
// }
