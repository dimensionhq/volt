pub mod app;
pub mod constants;
pub mod errors;
pub mod helper;
pub mod npm;
pub mod package;
pub mod volt_api;

use app::App;
use errors::VoltError;
use flate2::read::GzDecoder;
use futures_util::stream::FuturesUnordered;
use futures_util::StreamExt;
use git_config::file::GitConfig;
use git_config::parser::Parser;
use indicatif::ProgressBar;
use isahc::AsyncReadResponseExt;
use jwalk::WalkDir;
use lz4::Decoder;
use miette::DiagnosticResult;
use package::Package;
use reqwest::StatusCode;
use ssri::Algorithm;
use ssri::Integrity;
use std::borrow::Cow;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::env::temp_dir;
use std::ffi::OsStr;
use std::fs::read_to_string;
use std::fs::File;
use std::io::Cursor;
use std::io::Read;
use std::io::Write;
use std::path::Component;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tar::Archive;
use tokio::fs::create_dir_all;
use tokio::fs::hard_link;
use volt_api::VoltPackage;
use volt_api::VoltResponse;

use crate::constants::MAX_RETRIES;
use crate::volt_api::JSONVoltResponse;

/// decompress lz4 compressed json data
/// lz4 has the fastest decompression speeds
pub fn decompress(data: Vec<u8>) -> DiagnosticResult<Vec<u8>> {
    // initialize decoded data
    let mut decoded: Vec<u8> = Vec::new();

    // generate cursor (impl `Read`)
    let cursor = Cursor::new(data);

    // initialize a decoder
    let mut decoder = Decoder::new(cursor).map_err(VoltError::DecoderError)?;

    // decode and return data
    decoder
        .read_to_end(&mut decoded)
        .map_err(VoltError::DecoderError)?;

    Ok(decoded)
}

/// convert a JSONVoltResponse -> VoltResponse
pub fn convert(deserialized: JSONVoltResponse) -> DiagnosticResult<VoltResponse> {
    // initialize a hashmap to store the converted versions
    let mut converted_versions: HashMap<String, VoltPackage> = HashMap::new();

    // iterate through all listed dependencies of the latest version of the response
    for version in deserialized.versions.get(&deserialized.latest).unwrap() {
        // access data in the hashmap, not name@version
        let data = version.1;

        // @codemirror/state -> state
        let split = version
            .0
            .split("@")
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>();

        let mut package_name = String::new();

        if split.len() == 2 {
            package_name = split[0].to_string();
            if package_name.contains("/") {
                package_name = format!("@{}", package_name);
            }
        }

        // @codemirror/state@1.2.3 -> 1.2.3
        let package_version = version.0.split("@").last().unwrap();

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

    let mut final_res: HashMap<String, HashMap<String, VoltPackage>> = HashMap::new();

    final_res.insert(deserialized.latest.to_string(), converted_versions);

    Ok(VoltResponse {
        version: deserialized.latest,
        versions: final_res,
    })
}

// Get response from volt CDN
pub async fn get_volt_response(
    package_name: &String,
    hash: &String,
    package: Option<VoltPackage>,
) -> DiagnosticResult<VoltResponse> {
    // number of retries
    let mut retries = 0;

    // only 1 package, zero dependencies
    if package.is_some() {
        let package = package.as_ref().unwrap();

        let mut versions: HashMap<String, HashMap<String, VoltPackage>> = HashMap::new();

        let mut nested_versions: HashMap<String, VoltPackage> = HashMap::new();

        nested_versions.insert(
            format!("{}{}", package.name, package.version),
            package.clone(),
        );

        versions.insert(package.clone().version, nested_versions);

        return Ok(VoltResponse {
            version: package.clone().version,
            versions,
        });
    }
    // loop until MAX_RETRIES reached.
    loop {
        // get a response
        let mut response =
            isahc::get_async(format!("http://push-2105.5centscdn.com/{}.json", hash))
                .await
                .map_err(VoltError::NetworkError)?;

        // check the status of the response
        match response.status() {
            // 200 (OK)
            StatusCode::OK => {
                let mut buf: Vec<u8> = vec![];

                response
                    .copy_to(&mut buf)
                    .await
                    .map_err(VoltError::NetworkRecError)?;

                // decompress using lz4
                let decoded = decompress(buf)?;

                let deserialized: JSONVoltResponse =
                    serde_json::from_slice(&decoded).map_err(|_| VoltError::DeserializeError)?;

                let converted = convert(deserialized)?;

                return Ok(converted);
            }
            // 429 (TOO_MANY_REQUESTS)
            StatusCode::TOO_MANY_REQUESTS => Err(VoltError::TooManyRequests {
                url: format!("http://registry.voltpkg.com/{}", package_name),
                package_name: package_name.to_string(),
            })?,
            // 400 (BAD_REQUEST)
            StatusCode::BAD_REQUEST => Err(VoltError::BadRequest {
                url: format!("http://registry.voltpkg.com/{}", package_name),
                package_name: package_name.to_string(),
            })?,
            // 404 (NOT_FOUND)
            StatusCode::NOT_FOUND => {
                if retries == MAX_RETRIES {
                    Err(VoltError::PackageNotFound {
                        url: format!("http://registry.voltpkg.com/{}", package_name),
                        package_name: package_name.to_string(),
                    })?
                }
            }
            // Other Errors
            _ => {
                // Stop at MAX_RETRIES
                if retries == MAX_RETRIES {
                    Err(VoltError::NetworkUnknownError {
                        url: format!("http://registry.voltpkg.com/{}", package_name),
                        package_name: package_name.to_string(),
                        code: response.status().as_str().to_string(),
                    })?
                }
            }
        }

        // Increment no. retries
        retries += 1;
    }
}

pub async fn get_volt_response_multi(
    versions: &Vec<(String, String, String, Option<VoltPackage>)>,
    pb: &ProgressBar,
) -> Vec<DiagnosticResult<VoltResponse>> {
    versions
        .into_iter()
        .map(|(name, _, hash, package)| get_volt_response(&name, &hash, package.to_owned()))
        .collect::<FuturesUnordered<_>>()
        .inspect(|_| pb.inc(1))
        .collect::<Vec<DiagnosticResult<VoltResponse>>>()
        .await
}

#[cfg(windows)]
pub async fn hardlink_files(app: Arc<App>, src: PathBuf) {
    for entry in WalkDir::new(src) {
        let entry = entry.unwrap();

        if !entry.path().is_dir() {
            // index.js
            let entry = entry.path();

            let file_name = entry.file_name().unwrap().to_str().unwrap();

            // lib/index.js
            let path = format!("{}", &entry.display())
                .replace(r"\", "/")
                .replace(&app.volt_dir.display().to_string(), "");

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
                )
                .await
                .unwrap_or_else(|_| {
                    0;
                });
            }
        }
    }
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
pub async fn download_tarball(
    app: &App,
    package: &VoltPackage,
    secure: bool,
) -> DiagnosticResult<()> {
    let package_instance = package.clone();

    // @types/eslint
    if package_instance.name.starts_with('@') && package_instance.name.contains("/") {
        let package_directory_location = app
            .volt_dir
            .join(&package.name.split("/").collect::<Vec<&str>>()[0]);

        if !Path::new(&package_directory_location).exists() {
            create_dir_all(&package_directory_location)
                .await
                .map_err(VoltError::CreateDirError)?;
        }
    }

    // location of extracted package
    let loc = app.volt_dir.join(&package.name);

    // if package is not already installed
    if !Path::new(&loc).exists() {
        // Url to download tarball code files from
        let mut url = package_instance.tarball;
        // let registries = vec!["yarnpkg.com"];
        // let random_registry = registries.choose(&mut rand::thread_rng()).unwrap();

        // url = url.replace("npmjs.org", random_registry);

        if !secure {
            url = url.replace("https", "http")
        }

        // Get Tarball File
        let res = reqwest::get(url).await.unwrap();

        // Tarball bytes response
        let bytes: bytes::Bytes = res.bytes().await.unwrap();

        let algorithm;

        // there are only 2 supported algorithms
        // sha1 and sha512
        // so we can be sure that if it doesn't start with sha1, it's going to have to be sha512
        if package.integrity.starts_with("sha1") {
            algorithm = Algorithm::Sha1;
        } else {
            algorithm = Algorithm::Sha512;
        }

        // Verify If Bytes == (Sha 512 | Sha 1) of Tarball
        if package.integrity == App::calc_hash(&bytes, algorithm).unwrap() {
            // Create node_modules
            create_dir_all(&app.node_modules_dir).await.unwrap();

            // Delete package from node_modules
            let node_modules_dep_path = app.node_modules_dir.join(&package.name);

            // TODO: fix this
            // if node_modules_dep_path.exists() {
            //     remove_dir_all(&node_modules_dep_path).unwrap();
            // }

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

            extract_directory = extract_directory.join(format!(
                "{}-{}",
                package.clone().name,
                package.clone().version
            ));

            // Initialize tarfile decoder while directly passing in bytes

            let bytes = Arc::new(bytes);

            let bytes_ref = bytes.clone();

            let extract_directory_instance = extract_directory.clone();

            let node_modules_dep_path_instance = app.clone().node_modules_dir.clone();
            let pkg_name = package.clone().name;
            let pkg_name_instance = package.clone().name;

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
                                new_path.push(Component::Normal(OsStr::new(&pkg_name)));
                            } else {
                                new_path.push(component)
                            }
                        }

                        std::fs::create_dir_all(
                            node_modules_dep_path_instance
                                .to_path_buf()
                                .join(new_path.clone())
                                .parent()
                                .unwrap(),
                        )
                        .unwrap();

                        entry
                            .unpack(node_modules_dep_path_instance.to_path_buf().join(new_path))
                            .unwrap_or_else(|e| {
                                println!("{:?}", e);
                                std::process::exit(1);
                            });
                    }
                }),
                tokio::task::spawn_blocking(move || {
                    let gz_decoder = GzDecoder::new(&**bytes);

                    let mut archive = Archive::new(gz_decoder);

                    for entry in archive.entries().unwrap() {
                        let mut entry = entry.unwrap();
                        let path = entry.path().unwrap();
                        let mut new_path = PathBuf::new();

                        for component in path.components() {
                            if component.as_os_str() == "package" {
                                new_path.push(Component::Normal(OsStr::new(&pkg_name_instance)));
                            } else {
                                new_path.push(component)
                            }
                        }

                        std::fs::create_dir_all(
                            extract_directory_instance
                                .to_path_buf()
                                .join(new_path.clone())
                                .parent()
                                .unwrap(),
                        )
                        .unwrap();

                        entry
                            .unpack(extract_directory_instance.to_path_buf().join(new_path))
                            .unwrap_or_else(|e| {
                                println!("{:?}", e);
                                std::process::exit(1);
                            });
                    }
                })
            )
            .unwrap();
        } else {
            return Err(VoltError::ChecksumVerificationError)?;
        }
    }

    Ok(())
}

pub async fn download_tarball_create(
    _app: &App,
    package: &Package,
    name: &str,
) -> DiagnosticResult<String> {
    let file_name = format!("{}-{}.tgz", name, package.dist_tags.get("latest").unwrap());
    let temp_dir = temp_dir();

    if !Path::new(&temp_dir.join("volt")).exists() {
        std::fs::create_dir(Path::new(&temp_dir.join("volt")))
            .map_err(VoltError::CreateDirError)?;
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
    // if let Ok(hash) = App::calc_hash(&bytes::Bytes::from(bytes)) {
    //     // File exists, make sure it's not corrupted
    //     if hash
    //         == package
    //             .versions
    //             .get(package.dist_tags.get("latest").unwrap())
    //             .unwrap()
    //             .dist
    //             .shasum
    //     {
    //         return Ok(path_str);
    //     }
    // }

    let tarball = package_version.dist.tarball.replace("https", "http");

    let res = reqwest::get(tarball).await.unwrap();

    let bytes = res.bytes().await.unwrap();

    // App::calc_hash(&bytes)?;

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
/// Uses `gitoxide` to read from your git configuration.
pub fn get_git_config(app: &App, key: &str) -> Option<String> {
    match key {
        "user.name" => {
            let config_path = app.home_dir.join(".gitconfig");

            if !config_path.exists() {
                return None;
            } else {
                let data = read_to_string(config_path).ok()?;

                let config = GitConfig::from(Parser::try_from(data.as_str()).ok()?);
                let value = config.get_raw_value("user", None, "name").ok()?;

                return Some(String::from_utf8_lossy(&value).to_owned().to_string());
            }
        }
        "user.email" => {
            let config_path = app.home_dir.join(".gitconfig");

            if !config_path.exists() {
                return None;
            } else {
                let data = read_to_string(config_path).ok()?;

                let config = GitConfig::from(Parser::try_from(data.as_str()).ok()?);
                let value = config.get_raw_value("user", None, "email").ok()?;

                return Some(String::from_utf8_lossy(&value).to_owned().to_string());
            }
        }
        "repository.url" => {
            let remote_config_path = app.current_dir.join(".git").join("config");

            if !remote_config_path.exists() {
                let data = read_to_string(remote_config_path).ok()?;

                let config = GitConfig::from(Parser::try_from(data.as_str()).ok()?);
                let value = config.get_raw_value("remote", Some("origin"), "url").ok()?;

                return Some(String::from_utf8_lossy(&value).to_owned().to_string());
            } else {
                return None;
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

/// package all steps for installation into 1 convinient function.
pub async fn install_extract_package(
    app: &Arc<App>,
    package: &VoltPackage,
) -> DiagnosticResult<()> {
    // if there's an error (most likely a checksum verification error) while using insecure http, retry.
    if download_tarball(&app, &package, false).await.is_err() {
        // use https instead
        download_tarball(&app, &package, true)
            .await
            .unwrap_or_else(|_| {
                println!("failed to download tarball");
                std::process::exit(1);
            });
    }

    // generate the package's script
    generate_script(&app, package);

    // let directory = &app
    //     .volt_dir
    //     .join(package.version.clone())
    //     .join(package.name.clone());

    // let path = Path::new(directory.as_os_str());

    // hardlink_files(app.to_owned(), (&path).to_path_buf()).await;

    Ok(())
}
