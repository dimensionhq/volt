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

#[macro_use]
pub mod helper;
pub mod constants;
pub mod errors;
pub mod package;
pub mod scripts;
pub mod voltapi;

use crate::{
    cli::VoltConfig,
    core::{
        utils::constants::MAX_RETRIES,
        utils::voltapi::{VoltPackage, VoltResponse},
    },
};

use colored::Colorize;
use errors::VoltError;
// use flate2::read::GzDecoder;
use futures_util::{stream::FuturesUnordered, StreamExt};
use git_config::parser::parse_from_str;
use git_config::{file::GitConfig, parser::Parser};
use indicatif::ProgressBar;
use isahc::AsyncReadResponseExt;
use miette::{IntoDiagnostic, Result};
use package_spec::PackageSpec;
use reqwest::{Client, StatusCode};
use speedy::Readable;
use ssri::{Algorithm, Integrity};
use tar::Archive;

use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::read_to_string,
    io::{Cursor, Read, Write},
    path::PathBuf,
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
                name, requested, ..
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

    if let PackageSpec::Npm { name, .. } = package_spec {
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

pub fn decompress_tarball(gz_data: &[u8]) -> miette::Result<Vec<u8>> {
    // gzip RFC1952: a valid gzip file has an ISIZE field in the
    // footer, which is a little-endian u32 number representing the
    // decompressed size. This is ideal for libdeflate, which needs
    // preallocating the decompressed buffer.
    let isize = {
        let isize_start = gz_data.len() - 4;
        let isize_bytes: [u8; 4] = gz_data[isize_start..].try_into().into_diagnostic()?;
        u32::from_le_bytes(isize_bytes) as usize
    };

    let mut decompressor = libdeflater::Decompressor::new();

    let mut outbuf = vec![0; isize];
    decompressor
        .gzip_decompress(gz_data, &mut outbuf)
        .into_diagnostic()?;

    Ok(outbuf)
}

/// downloads and extracts tarball file from package
pub async fn fetch_tarball(
    config: &VoltConfig,
    package: &VoltPackage,
    state: State,
) -> Result<(bytes::Bytes)> {
    // Recieve the tarball from the npm registry
    let response = state
        .http_client
        .get(&package.tarball)
        .send()
        .await
        .into_diagnostic()?
        .bytes()
        .await
        .into_diagnostic()?;

    Ok(response)
}

/// Gets a config key from git using the git cli.
/// Uses `gitoxide` to read from your git configuration.
pub fn get_git_config(config: &VoltConfig, key: &str) -> Result<Option<String>> {
    fn get_git_config_value_if_exists(
        config: &VoltConfig,
        section: &str,
        subsection: Option<&str>,
        key: &str,
    ) -> Result<Option<String>> {
        let config_path = config.home()?.join(".gitconfig");

        if config_path.exists() {
            let data = read_to_string(config_path).into_diagnostic()?;

            let parser = parse_from_str(&data).map_err(|err| VoltError::GitConfigParseError {
                error_text: err.to_string(),
            })?;
            let config = GitConfig::from(parser);
            let value = config.get_raw_value(section, subsection, key).ok();

            Ok(value.map(|v| String::from_utf8_lossy(&v).into_owned()))
        } else {
            Ok(None)
        }
    }

    match key {
        "user.name" => get_git_config_value_if_exists(config, "user", None, "name"),
        "user.email" => get_git_config_value_if_exists(config, "user", None, "email"),
        "repository.url" => get_git_config_value_if_exists(config, "remote", Some("origin"), "url"),
        _ => Ok(None),
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
pub fn generate_script(config: &VoltConfig, package: &VoltPackage) {
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
pub fn generate_script(config: &VoltConfig, package: &VoltPackage) {
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

pub fn check_peer_dependency(_package_name: &str) -> bool {
    false
}

pub fn verify_existing_installation(
    package: &VoltPackage,
    config: &VoltConfig,
) -> miette::Result<bool> {
    let cacache_key = package.cacache_key();
    let volt_home = config.volt_home().unwrap();

    Ok(cacache::exists_sync(
        &volt_home,
        &Integrity::from(&package.integrity),
    ))
}

pub fn verify_checksum(
    response: &bytes::Bytes,
    target_integrity: String,
) -> miette::Result<(bool, Option<String>)> {
    // begin
    let algorithm;

    // there are only 2 supported algorithms
    // sha1 and sha512
    // so we can be sure that if it doesn't start with sha1, it's going to have to be sha512
    if target_integrity.starts_with("sha1") {
        algorithm = Algorithm::Sha1;
    } else {
        algorithm = Algorithm::Sha512;
    }

    let calculated_checksum = VoltConfig::calc_hash(&response, algorithm)?;

    if calculated_checksum == target_integrity {
        return Ok((true, None));
    } else {
        return Ok((false, Some(calculated_checksum)));
    }
}

/// Install a JavaScript package.
pub async fn install_package(
    config: &VoltConfig,
    package: &VoltPackage,
    state: State,
) -> Result<()> {
    // Check if the package is already installed
    if !verify_existing_installation(package, config)? {
        // fetch the tarball from the registry
        let response = fetch_tarball(config, package, state).await?;

        // verify the checksum
        // (checksum is valid, calculated checksum)
        let (verified, checksum) = verify_checksum(&response, package.integrity.clone())?;

        if verified {
            // decompress gzipped response
            let decompressed_response = decompress_tarball(&response)?;

            // extract the tarball
            extract_tarball(decompressed_response, &package, &config)?;
        } else {
            // TODO: handle checksum failure
        }
    }

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
            name, requested, ..
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

pub fn extract_tarball(
    data: Vec<u8>,
    package: &VoltPackage,
    config: &VoltConfig,
) -> miette::Result<()> {
    // Generate the tarball archive given the decompressed bytes
    let mut node_archive = Archive::new(Cursor::new(data));

    // extract to both the global store + node_modules (in the case of them using the pnpm linking algorithm)
    let mut cas_file_map: HashMap<String, Integrity> = HashMap::new();

    // Add package's directory to list of created directories
    let mut created_directories: Vec<PathBuf> = vec![];

    for entry in node_archive.entries().into_diagnostic()? {
        let mut entry = entry.into_diagnostic()?;

        // Read the contents of the entry
        let mut buffer = vec![0; entry.size() as usize];
        entry.read_to_end(&mut buffer).into_diagnostic()?;

        let entry_path_string = entry
            .path()
            .into_diagnostic()?
            .to_str()
            .expect("valid utf-8")
            .to_string();

        // Remove `package/` from `package/lib/index.js`
        let cleaned_entry_path_string =
            if let Some(i) = entry_path_string.char_indices().position(|(_, c)| c == '/') {
                &entry_path_string[i + 1..]
            } else {
                &entry_path_string[..]
            };

        // Create the path to the local .volt directory
        let mut package_directory = config.node_modules()?.join(VoltConfig::VOLT_HOME);

        // Add package's directory to it
        package_directory.push(package.directory_name());

        // push node_modules/.volt/send@0.17.2 to the list (because we created it in the previous step)
        created_directories.push(package_directory.clone());

        // Add the cleaned path to the package's directory
        let mut entry_path = package_directory.clone();

        entry_path.push("node_modules");

        entry_path.push(&package.name);

        entry_path.push(cleaned_entry_path_string);

        // Get the entry's parent
        let entry_path_parent = entry_path.parent().unwrap();

        // If we haven't created this directory yet, create it
        if !created_directories.iter().any(|p| p == entry_path_parent) {
            created_directories.push(entry_path_parent.to_path_buf());
            std::fs::create_dir_all(entry_path_parent).into_diagnostic()?;
        }

        let mut file_path = package_directory.join("node_modules");

        file_path.push(package.name.clone());

        file_path.push(cleaned_entry_path_string);

        // Write the contents to node_modules
        let mut file = std::fs::File::create(&file_path).unwrap();

        file.write_all(&buffer);

        // Write the contents of the entry into the content-addressable store located at `app.volt_dir`
        // We get a hash of the file
        let sri = cacache::write_hash_sync(&config.volt_home()?, &buffer).into_diagnostic()?;

        // Insert the name of the file and map it to the hash of the file
        cas_file_map.insert(entry_path_string, sri);
    }

    // Write the file, shasum map to the content-addressable store
    cacache::write_sync(
        &config.volt_home()?,
        &package.cacache_key(),
        serde_json::to_string(&cas_file_map).into_diagnostic()?,
    )
    .into_diagnostic()?;

    Ok(())
}
