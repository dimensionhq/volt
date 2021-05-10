use std::{io, process};

use crate::constants;
use colored::Colorize;
use constants::{
    about, add_error, add_help, help, init_help, install_help, remove_error, remove_help,
};

const __VERSION__: &str = "v1.0.0";

pub fn initialize() -> Vec<String> {
    // Initialize And Get Args
    enable_ansi_support().unwrap();
    std::env::args().collect()
}

pub fn display_help(args: &Vec<String>) -> &String {
    if args.len() == 1 {
        about();
    } else if args.len() == 2 {
        let command: &str = args[1].as_str();
        match command {
            "--version" => println!("{}", format!("volt {}", __VERSION__.bright_green().bold())),
            "init" => {}
            "install" => {}
            "add" => add_error(),
            "remove" => remove_error(),
            "--help" => help(),
            &_ => {}
        }
    } else if args.len() == 3 {
        let command: &str = args[1].as_str();
        if args[2].as_str().starts_with("--") {
            let flag: &str = args[2].as_str();
            if flag == "--help" {
                match command {
                    "init" => init_help(),
                    "install" => install_help(),
                    "add" => add_help(),
                    "remove" => remove_help(),
                    &_ => {}
                }
            }
        }
    }

    &args[1]
}

pub fn handle_invalid_command(command: &str) {
    println!(
        "{}",
        format!("{} Is Not A Valid Command!", command.bright_red())
    );
}

pub fn get_arguments(args: &Vec<String>) -> (Vec<String>, Vec<String>) {
    let command: &str = &args[1];
    let mut flags: Vec<String> = vec![];
    let mut packages: Vec<String> = vec![];

    for arg in 0..args.len() {
        if arg > 1 {
            if command == "init" || command == "install" || command == "add" || command == "remove"
            {
                if args[arg].starts_with("--") {
                    flags.push(args[arg].clone());
                } else {
                    packages.push(args[arg].clone());
                }
            }
        }
    }

    (flags, packages)
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
