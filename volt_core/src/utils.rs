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

use std::fs::create_dir;
use std::fs::create_dir_all;
use std::io::Read;
use std::io::{self, Write};
use std::path::Path;
use std::process;
use std::process::exit;
use std::{borrow::Cow, path::PathBuf};
use std::{env::temp_dir, fs::File};

use anyhow::Result;
use chttp::ResponseExt;
use colored::Colorize;

use crate::app::App;
use crate::classes::voltapi::{VoltPackage, VoltResponse};

#[cfg(windows)]
pub static PROGRESS_CHARS: &str = "=> ";

#[cfg(unix)]
pub static PROGRESS_CHARS: &str = "▰▰▱";

lazy_static! {
    pub static ref ERROR_TAG: String = "error".red().bold().to_string();
}

// pub fn get_dependencies_recursive(
//     dep: &str,
//     pkgname: &str,
//     packages: &std::collections::HashMap<String, VoltPackage>,
// ) {
//     for (name, package) in &packages.clone() {
//         if name == dep {
//             if package.dependencies != None {
//                 let dependencies = package.dependencies.clone().unwrap();
//                 println!("{}: {:?}", name, dependencies);
//                 if dependencies.len() != 0 {
//                     let user_profile = std::env::var("USERPROFILE").unwrap();
//                     let volt_dir_loc = format!(r"{}\.volt", user_profile);
//                     let volt_dir = Path::new(&volt_dir_loc);
//                     let name_dir_loc = volt_dir.join(name);
//                     let name_dir = Path::new(&name_dir_loc);
//                     let dep_nodemodules_dir = name_dir.join("node_modules");
//                     if !dep_nodemodules_dir.clone().exists() {
//                         std::fs::create_dir(dep_nodemodules_dir.clone()).unwrap();
//                     }
//                     for dep in dependencies.clone() {
//                         let volt_dep_dir = volt_dir.join(
//                             dep.clone()
//                                 .replace("/", "__")
//                                 .replace("@", "")
//                                 .replace(".", "_"),
//                         );
//                         let dep_dir = dep_nodemodules_dir
//                             .join(dep.replace("/", "__").replace("@", "").replace(".", "_"));
//                         // println!(
//                         //     "dir: {:?}\ndir: {:?}",
//                         //     volt_dep_dir,
//                         //     dep_nodemodules_dir.clone()
//                         // );
//                         if !dep_dir.exists() {
//                             // std::fs::create_dir(dep_dir.clone()).unwrap();
//                             copy(
//                                 volt_dep_dir,
//                                 dep_nodemodules_dir.clone(),
//                                 &CopyOptions::new(),
//                             )
//                             .unwrap();
//                         }
//                     }
//                 }
//                 for dep in dependencies {
//                     get_dependencies_recursive(dep.as_str(), pkgname, &packages)
//                 }
//             }
//         }
//     }
// }

pub fn get_dependencies_recursive(
    pkgname: &str,
    volt_dir: &Path,
    package_dir: PathBuf,
    packages: &std::collections::HashMap<String, VoltPackage>,
) {
    let mut dependencies = vec![];
    for (name, package) in packages {
        if name == pkgname {
            let dependency_list = &package.dependencies;
            if dependency_list != &None {
                for dep in dependency_list.clone().unwrap() {
                    dependencies.push(dep);
                }
            }
        }
    }

    if !dependencies.is_empty() {
        let node_modules_dir = package_dir.join("node_modules");
        if !node_modules_dir.exists() {
            std::fs::create_dir(node_modules_dir.clone()).unwrap();
        }
        for dep in dependencies {
            // println!("dep: {}", dep);
            let volt_dep_dir = volt_dir.join(dep.clone().replace("/", "__").replace("@", ""));
            let dep_dir = node_modules_dir.join(dep.clone().replace("/", "__").replace("@", ""));
            if !dep_dir.exists() {
                // copy(volt_dep_dir, node_modules_dir.clone(), &CopyOptions::new()).unwrap();
                create_symlink(
                    volt_dep_dir.as_os_str().to_str().unwrap().to_string(),
                    dep_dir.as_os_str().to_str().unwrap().to_string(),
                )
                .unwrap();
            }

            // let node_modules_dep_path = std::env::current_dir().unwrap().join(format!(
            //     r"node_modules\{}",
            //     dep.replace("/", "__").replace("@", "").replace(".", "_")
            // ));

            // if !node_modules_dep_path.exists() {
            //     create_symlink(
            //         volt_dir_file_path.as_os_str().to_str().unwrap().to_string(),
            //         node_modules_dep_path
            //             .as_os_str()
            //             .to_str()
            //             .unwrap()
            //             .to_string(),
            //     )?;
            // }
            get_dependencies_recursive(dep.clone().as_str(), volt_dir, dep_dir, packages);
        }
    }
}

pub fn create_dep_symlinks(
    pkg_name: &str,
    packages: std::collections::HashMap<String, VoltPackage>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
    Box::pin(async move {
        let user_profile;
        let volt_dir_loc;
        if cfg!(windows) {
            user_profile = std::env::var("USERPROFILE").unwrap();
            volt_dir_loc = format!(r"{}\.volt", user_profile);
        } else {
            user_profile = std::env::var("HOME").unwrap();
            volt_dir_loc = format!(r"{}/.volt", user_profile);
        }
        let volt_dir = Path::new(&volt_dir_loc);
        let package_dir = volt_dir.join(pkg_name);
        get_dependencies_recursive(pkg_name, volt_dir, package_dir.clone(), &packages);

        let node_modules_dep_path =
            std::env::current_dir()?.join(format!(r"node_modules\{}", pkg_name));
        create_symlink(
            package_dir.as_os_str().to_str().unwrap().to_string(),
            node_modules_dep_path
                .as_os_str()
                .to_str()
                .unwrap()
                .to_string(),
        )?;
        Ok(())
        // let mut package_list: Vec<VoltPackage> = vec![];
        // let mut dependencies = vec![];
        // let user_profile = std::env::var("USERPROFILE")?;

        // for (_, object) in &packages {
        //     package_list.push(object.clone());
        // }

        // for package in package_list.clone() {
        //     // println!("object: {:?}", package);
        //     if package.dependencies != None {
        //         if package.dependencies.clone().unwrap().len() != 0 {
        //             for dep in package.dependencies.unwrap() {
        //                 if !dependencies.contains(&dep) {
        //                     dependencies.push(dep);
        //                 }
        //             }
        //         }
        //     }
        //     // dependencies.push(package.name);
        // }

        // println!("dep: {:?}", dependencies);

        // for dep in dependencies.clone() {
        //     let mut inside_deps = vec![];
        //     for package in package_list.clone() {
        //         if package.name == dep {
        //             if package.dependencies != None {
        //                 if package.dependencies.clone().unwrap().len() != 0 {
        //                     for dep in package.dependencies.unwrap() {
        //                         if !inside_deps.contains(&dep) {
        //                             inside_deps.push(dep);
        //                         }
        //                     }
        //                 }
        //             }
        //         }
        //     }
        //     println!("inside dep: {:?}", inside_deps);
        //     if inside_deps.len() != 0 {
        //         for inside_dep in inside_deps {
        //             let loc_dep_dir = format!(
        //                 r"{}\.volt\{}\node_modules\{}",
        //                 user_profile,
        //                 dep.replace("/", "__").replace("@", "").replace(".", "_"),
        //                 inside_dep
        //             );
        //             let node_modules_loc = format!(
        //                 r"{}\.volt\{}\node_modules",
        //                 user_profile,
        //                 dep.replace("/", "__").replace("@", "").replace(".", "_")
        //             );
        //             let node_modules_dir = Path::new(&node_modules_loc);
        //             let dep_dir = Path::new(&loc_dep_dir);
        //             let loc = format!(r"{}\.volt\{}", user_profile, inside_dep);
        //             let volt_dep_dir = Path::new(&loc);
        //             if !node_modules_dir.exists() {
        //                 std::fs::create_dir(node_modules_dir)?;
        //             }
        //             if !dep_dir.exists() && volt_dep_dir.exists() {
        //                 copy(volt_dep_dir, node_modules_dir, &CopyOptions::new())?;
        //             }
        //         }
        //     }
        //     // let response: Package = get_yarn_response(dep).await;
        //     // let deps: Option<Vec<String>> = Some(response.versions[&response.dist_tags.latest].dependencies.keys().cloned().collect());
        //     // Box::pin(create_dep_symlinks(dep_dir.as_str(), deps)).await.unwrap();
        // }

        // for dep in dependencies {
        //     let volt_dir_dep_loc = format!(
        //         r"{}\.volt\{}",
        //         user_profile,
        //         dep.replace("/", "__").replace("@", "").replace(".", "_")
        //     );
        //     let volt_dir_file_path = Path::new(&volt_dir_dep_loc);
        // let node_modules_dep_path = std::env::current_dir()?.join(format!(
        //     r"node_modules\{}",
        //     dep.replace("/", "__").replace("@", "").replace(".", "_")
        // ));
        // println!(
        //     "dir: {:?}\ndir: {:?}",
        //     volt_dir_file_path, node_modules_dep_path
        // );
        // if !node_modules_dep_path.exists() {
        //     create_symlink(
        //         volt_dir_file_path.as_os_str().to_str().unwrap().to_string(),
        //         node_modules_dep_path
        //             .as_os_str()
        //             .to_str()
        //             .unwrap()
        //             .to_string(),
        //     )?;
        // }
        // }

        // Ok(())
    })
}

// pub fn create_dep_symlinks_old(
//     _current_dep_dir: &str,
//     packages: std::collections::HashMap<String, VoltPackage>,
// ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
//     Box::pin(async move {
//         let mut package_list: Vec<VoltPackage> = vec![];
//         let mut dependencies = vec![];
//         let user_profile = std::env::var("USERPROFILE")?;

//         for (_, object) in &packages {
//             package_list.push(object.clone());
//         }

//         for package in package_list.clone() {
//             // println!("object: {:?}", package);
//             if package.dependencies != None {
//                 if package.dependencies.clone().unwrap().len() != 0 {
//                     for dep in package.dependencies.unwrap() {
//                         if !dependencies.contains(&dep) {
//                             dependencies.push(dep);
//                         }
//                     }
//                 }
//             }
//             dependencies.push(package.name);
//         }

//         for dep in dependencies.clone() {
//             let mut inside_deps = vec![];
//             for package in package_list.clone() {
//                 if package.name == dep {
//                     if package.dependencies != None {
//                         if package.dependencies.clone().unwrap().len() != 0 {
//                             for dep in package.dependencies.unwrap() {
//                                 if !inside_deps.contains(&dep) {
//                                     inside_deps.push(dep);
//                                 }
//                             }
//                         }
//                     }
//                 }
//             }
//             // println!("dep: {:?}", inside_deps);
//             if inside_deps.len() != 0 {
//                 for inside_dep in inside_deps {
//                     let loc_dep_dir = format!(
//                         r"{}\.volt\{}\node_modules\{}",
//                         user_profile,
//                         dep.replace("/", "__").replace("@", "").replace(".", "_"),
//                         inside_dep
//                     );
//                     let node_modules_loc = format!(
//                         r"{}\.volt\{}\node_modules",
//                         user_profile,
//                         dep.replace("/", "__").replace("@", "").replace(".", "_")
//                     );
//                     let node_modules_dir = Path::new(&node_modules_loc);
//                     let dep_dir = Path::new(&loc_dep_dir);
//                     let loc = format!(r"{}\.volt\{}", user_profile, inside_dep);
//                     let volt_dep_dir = Path::new(&loc);
//                     if !node_modules_dir.exists() {
//                         std::fs::create_dir(node_modules_dir)?;
//                     }
//                     if !dep_dir.exists() && volt_dep_dir.exists() {
//                         copy(volt_dep_dir, node_modules_dir, &CopyOptions::new())?;
//                     }
//                 }
//             }
//             // let response: Package = get_yarn_response(dep).await;
//             // let deps: Option<Vec<String>> = Some(response.versions[&response.dist_tags.latest].dependencies.keys().cloned().collect());
//             // Box::pin(create_dep_symlinks(dep_dir.as_str(), deps)).await.unwrap();
//         }

//         for dep in dependencies {
//             let volt_dir_dep_loc = format!(
//                 r"{}\.volt\{}",
//                 user_profile,
//                 dep.replace("/", "__").replace("@", "").replace(".", "_")
//             );
//             let volt_dir_file_path = Path::new(&volt_dir_dep_loc);
//             let node_modules_dep_path = std::env::current_dir()?.join(format!(
//                 r"node_modules\{}",
//                 dep.replace("/", "__").replace("@", "").replace(".", "_")
//             ));
//             println!(
//                 "dir: {:?}\ndir: {:?}",
//                 volt_dir_file_path, node_modules_dep_path
//             );
//             if !node_modules_dep_path.exists() {
//                 create_symlink(
//                     volt_dir_file_path.as_os_str().to_str().unwrap().to_string(),
//                     node_modules_dep_path
//                         .as_os_str()
//                         .to_str()
//                         .unwrap()
//                         .to_string(),
//                 )?;
//             }
//         }

//         Ok(())
//     })
// }

// pub async fn get_yarn_response(package_name: String) -> Package {
//     let response = reqwest::get(format!("http://registry.yarnpkg.com/{}", package_name))
//         .await
//         .unwrap_or_else(|e| {
//             println!("{} {}", "error".bright_red(), e);
//             std::process::exit(1);
//         })
//         .text()
//         .await
//         .unwrap_or_else(|e| {
//             println!("{} {}", "error".bright_red(), e);
//             std::process::exit(1);
//         });

//     let data = serde_json::from_str::<Package>(&response).unwrap_or_else(|e| {
//         println!("{} {}", "error".bright_red(), e);
//         std::process::exit(1);
//     });

//     data
// }

// Gets response from volt CDN
pub async fn get_volt_response(package_name: String) -> VoltResponse {
    let response = chttp::get_async(format!("http://volt-api.b-cdn.net/{}.json", package_name))
        .await
        .unwrap_or_else(|_| {
            println!("{} That package does not exist yet!", "error".bright_red(),);
            std::process::exit(1);
        })
        .text_async()
        .await
        .unwrap_or_else(|_| {
            println!("{} That package does not exist yet!", "error".bright_red());
            std::process::exit(1);
        });

    serde_json::from_str::<VoltResponse>(&response).unwrap_or_else(|_| {
        println!("{} Failed to parse response!", "error".bright_red());
        std::process::exit(1);
    })
}

/// downloads tarball file from package
pub async fn download_tarball(_app: &App, package: &VoltPackage) -> Result<String> {
    let name = &package.name.replace("/", "^^");
    let file_name = format!("{}@{}.tgz", name, package.version);
    let temp_dir = temp_dir();

    if !Path::new(&temp_dir.join("volt")).exists() {
        std::fs::create_dir(Path::new(&temp_dir.join("volt")))?;
    }

    if name.starts_with("@") && name.contains("^^") {
        let package_dir_loc;

        if cfg!(windows) {
            // Check if C:\Users\username\.volt exists
            package_dir_loc = format!(
                r"{}\.volt\{}",
                std::env::var("USERPROFILE").unwrap(),
                name.split("^^").collect::<Vec<&str>>()[0]
            );
        } else {
            // Check if ~/.volt\packagename exists
            package_dir_loc = format!(
                r"{}\.volt\{}",
                std::env::var("HOME").unwrap(),
                name.split("^^").collect::<Vec<&str>>()[0]
            );
        }

        if !Path::new(&package_dir_loc).exists() {
            create_dir_all(&package_dir_loc).unwrap();
        }
    }

    let path;

    if cfg!(windows) {
        path = temp_dir.join(format!(r"volt\{}", file_name));
    } else {
        path = temp_dir.join(format!(r"volt/{}", file_name));
    }

    let path_str = path.to_string_lossy().to_string();

    // Corrupt tar files may cause issues
    if let Ok(hash) = App::calc_hash(&path) {
        // File exists, make sure it's not corrupted
        if hash == package.sha1 {
            return Ok(path_str);
        }
    }

    let tarball = package.tarball.replace("https", "http");

    let response = chttp::get_async(tarball).await?;

    let body = response.into_body();

    let mut file = File::create(&path)?;

    file.write_all(
        body.bytes()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
            .as_slice(),
    )?;

    App::calc_hash(&path)?;

    Ok(path_str)
}

pub fn get_basename(path: &'_ str) -> Cow<'_, str> {
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

    return Ok(());
}

/// Create a junction / hard symlink to a directory
#[cfg(windows)]
pub fn create_symlink(original: String, link: String) -> Result<()> {
    junction::create(original, link)?;
    Ok(())
}

// Unix functions
#[cfg(unix)]
pub fn enable_ansi_support() -> Result<(), u32> {
    Ok(())
}

/// Create a symlink to a directory
#[cfg(unix)]
pub fn create_symlink<P: AsRef<Path>, Q: AsRef<Path>>(original: P, link: Q) -> Result<()> {
    std::os::unix::fs::symlink(original, link).context("Unable to symlink directory")
}

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

pub fn check_peer_dependency(_package_name: &String) -> bool {
    return false;
}
