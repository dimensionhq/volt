use crate::classes::package::Package;
use dirs::home_dir;
use flate2::read::GzDecoder;
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    env,
    fs::File,
    io::{self, Write},
    path::Path,
    process,
};
use tar::Archive;

pub struct App {
    pub current_dir: Box<Path>,
    pub home_dir: Box<Path>,
    pub volt_dir: Box<Path>,
}

pub fn initialize() -> (App, Vec<String>) {
    // Initialize And Get Args
    enable_ansi_support().unwrap();

    let current_dir = env::current_dir().unwrap().into_boxed_path();
    let home_dir = home_dir()
        .map(|dir| dir.into_boxed_path())
        .unwrap_or_else(|| current_dir.clone());

    let volt_dir = home_dir.join(".volt").into_boxed_path();

    let app = App {
        current_dir,
        home_dir,
        volt_dir,
    };

    (app, std::env::args().collect())
}

pub fn get_arguments(args: &Vec<String>) -> (Vec<String>, Vec<String>) {
    let mut flags: Vec<String> = vec![];
    let mut packages: Vec<String> = vec![];

    for arg in 0..args.len() {
        if arg > 1 {
            if args[arg].starts_with("--") || args[arg].starts_with("-") {
                flags.push(args[arg].clone());
            } else {
                packages.push(args[arg].clone());
            }
        }
    }

    (flags, packages)
}

/// downloads tarball file from package
pub async fn download_tarball(app: &App, package: &Package) -> String {
    let latest_version = &package.dist_tags.latest;
    let name = &package.name;
    let tarball = &package.versions[latest_version].dist.tarball;

    let mut response = reqwest::get(tarball).await.unwrap();
    let total_length = response.content_length().unwrap();
    let progress_bar = ProgressBar::new(total_length);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .progress_chars("=>-"),
    );

    let file_name = format!("{}-{}.tgz", name, latest_version);

    let path = app.volt_dir.join(file_name);
    let path_str = path.to_string_lossy().to_string();

    // Placeholder buffer
    let mut file = File::create(path).unwrap();

    while let Some(chunk) = response.chunk().await.unwrap() {
        progress_bar.inc(chunk.len() as u64);
        let _ = file.write(&*chunk);
    }

    progress_bar.finish();

    path_str
}

pub fn extract_tarball(file_path: &str, package: &Package) -> Result<(), std::io::Error> {
    let path = Path::new(file_path);
    let tar_gz = File::open(path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack("node_modules")?;
    if !Path::new(&format!(r"node_modules/{}", package.name)).exists() {
        std::fs::rename(
            r"node_modules/package",
            format!(r"node_modules/{}", package.name),
        )?;
    } else {
        let loc = format!(r"node_modules/{}/package.json", package.name);
        let file_contents = std::fs::read_to_string(loc).unwrap();
        let json_file: serde_json::Value = serde_json::from_str(file_contents.as_str()).unwrap();
        let version = json_file["version"].as_str().unwrap();
        if version != package.dist_tags.latest {
            // Update dependencies
        }
    }
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
