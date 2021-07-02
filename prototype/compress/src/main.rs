use colored::Colorize;
use indicatif::ProgressStyle;
use jwalk::WalkDir;
use std::ffi::OsStr;
use std::fs::{read_dir, read_to_string, remove_file};
use std::io::Write;
use std::os::windows::prelude::MetadataExt;
use std::time::Instant;

fn main() {
    // esbuild.cmd --minify --allow-overwrite --outfile=package.json .\package.json
    let start = Instant::now();
    let walkdir = WalkDir::new("node_modules");
    let mut old_size: u64 = 0;

    let spinner = indicatif::ProgressBar::new_spinner();
    spinner.set_style(ProgressStyle::default_spinner().template(&format!(
        "{{spinner}} {}",
        "Searching For Empty Files & Folders".bright_magenta()
    )));
    spinner.enable_steady_tick(100);

    let mut files: Vec<String> = vec![];
    let mut folders: Vec<String> = vec![];

    for e in walkdir {
        let e = e.unwrap();

        if e.path().is_file() {
            let file_size = &e.metadata().unwrap().file_size();

            if file_size == &0
                || &e
                    .path()
                    .extension()
                    .unwrap_or(OsStr::new(""))
                    .to_str()
                    .unwrap()
                    == &"md"
                || &e
                    .path()
                    .extension()
                    .unwrap_or(OsStr::new(""))
                    .to_str()
                    .unwrap()
                    == &"json"
                || e.path().file_name().unwrap().to_str().unwrap() == "LICENSE"
                || e.path().file_name().unwrap().to_str().unwrap() == "license"
                || e.path().file_name().unwrap().to_str().unwrap() == "README"
            {
                files.push(e.path().display().to_string());
            }

            old_size += *file_size as u64;
        } else {
            if read_dir(format!("{}", e.path().display())).unwrap().count() == 0 {
                folders.push(e.path().display().to_string());
            }
        }
    }

    spinner.finish_and_clear();

    println!("Optimizing Files");

    for f in files.clone() {
        if f.ends_with("json") {
            let data = read_to_string(f.to_string()).unwrap();
            let minified = minifier::json::minify(data.as_str());
            let mut file = std::fs::File::create(f).unwrap();
            file.write(minified.as_bytes()).unwrap();
        }
    }

    for f in files {
        if !f.ends_with("json") {
            remove_file(&f).unwrap();
        }
    }

    let mut end_size: u64 = 0;

    for file in WalkDir::new("node_modules") {
        let file = file.unwrap();
        if file.path().is_file() {
            end_size += file.metadata().unwrap().file_size();
        }
    }

    println!(
        "{}: cleaned node_modules saving {} Mb in {:.4} seconds",
        "success".bright_green().bold(),
        ((old_size - end_size) / 1024 / 1024),
        start.elapsed().as_secs_f32().to_string().bright_blue(),
    )
}
