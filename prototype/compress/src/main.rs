use indicatif::ProgressBar;
use minifier::{css, js, json};
use std::fs::read_to_string;
use std::path::Path;
use std::time::Instant;
use std::{ffi::OsStr, io::Write};
use walkdir::WalkDir;

fn get_extension_from_filename(filename: &str) -> Option<&str> {
    Path::new(filename).extension().and_then(OsStr::to_str)
}

fn main() {
    let start = Instant::now();
    let files = WalkDir::new("../../node_modules");

    for entry in files {
        let entry = entry.unwrap();

        let path = entry.path().display().to_string();

        if entry.file_type().is_file() {
            let data = read_to_string(&path).unwrap();
            match get_extension_from_filename(path.as_str()) {
                Some(value) => match value {
                    ".json" => {
                        let minified = json::minify(data.as_str());
                        let mut f = std::fs::OpenOptions::new()
                            .write(true)
                            .truncate(true)
                            .open(path)
                            .unwrap();

                        f.write(minified.as_bytes()).unwrap();
                    }
                    ".css" => {
                        let minified = css::minify(data.as_str()).unwrap();
                        let mut f = std::fs::OpenOptions::new()
                            .write(true)
                            .truncate(true)
                            .open(path)
                            .unwrap();

                        f.write(minified.as_bytes()).unwrap();
                    }
                    _ => {
                        let minified = js::minify(data.as_str());

                        let mut f = std::fs::OpenOptions::new()
                            .write(true)
                            .truncate(true)
                            .open(path)
                            .unwrap();

                        f.write(minified.as_bytes()).unwrap();
                    }
                },
                None => {
                    continue;
                }
            }
        }
    }

    println!("{}", start.elapsed().as_secs_f32());
}
