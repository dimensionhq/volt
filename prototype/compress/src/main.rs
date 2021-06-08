use std::io::Read;
use std::io::Write;
use std::process::Command;
use std::process::Stdio;
use std::time::Instant;

use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::path::Path;
use walkdir::WalkDir;

use flate2::write::GzEncoder;
use flate2::Compression;

enum SourceType {
    Css,
    Js,
    Json,
    Html,
    Unknown,
}

use std::{fs, io, path::PathBuf};

fn dir_size(path: impl Into<PathBuf>) -> io::Result<u64> {
    fn dir_size(mut dir: fs::ReadDir) -> io::Result<u64> {
        dir.try_fold(0, |acc, file| {
            let file = file?;
            let size = match file.metadata()? {
                data if data.is_dir() => dir_size(fs::read_dir(file.path())?)?,
                data => data.len(),
            };
            Ok(acc + size)
        })
    }

    dir_size(fs::read_dir(path.into())?)
}

fn minifiable<P: AsRef<Path>>(path: P) -> Option<SourceType> {
    let recognized_file_types = vec!["js", "ts", ".d.ts"];

    let ext = path.as_ref().extension()?;
    if ext == "css" {
        Some(SourceType::Css)
    } else if recognized_file_types.contains(&ext.to_str().unwrap()) {
        Some(SourceType::Js)
    } else if ext == "json" {
        Some(SourceType::Json)
    } else if ext == "html" {
        Some(SourceType::Html)
    } else {
        Some(SourceType::Unknown)
    }
}

fn minify_file(path: String, src_ty: SourceType) {
    // Minify
    match src_ty {
        SourceType::Css => {}
        SourceType::Json => {
            let mut buf = String::new();

            // Open File
            File::open(&path).unwrap().read_to_string(&mut buf).unwrap();

            let minified: String = minifier::json::minify(&buf);

            File::create(&path)
                .unwrap()
                .write_all(minified.as_bytes())
                .unwrap();
        }
        SourceType::Js => {
            minify_js(path);
        }
        SourceType::Html => {}
        SourceType::Unknown => {}
    }
}

fn main() {
    let start = Instant::now();
    let files = WalkDir::new("node_modules");
    let old_size = dir_size("node_modules").unwrap();

    let to_minify: Vec<_> = files
        .into_iter()
        // Skip filesystem errors rather than panicking
        .filter_map(Result::ok)
        // Only look at files, not dirs or symlinks
        .filter(|entry| entry.file_type().is_file())
        // If something's minifiable, determine its type. If not, skip it.
        .filter_map(|entry| {
            let src_ty = minifiable(entry.file_name())?;
            Some((entry, src_ty))
        })
        .collect();

    let pb = ProgressBar::new(to_minify.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .progress_chars("=> "),
    );

    for (entry, src_ty) in to_minify {
        let path = entry.path();

        let name = path.to_string_lossy().into_owned();
        pb.set_message(name.clone());

        minify_file(name, src_ty);

        // Either way, we're done with this file
        pb.inc(1);
    }

    pb.finish_and_clear();

    let msg = format!(
        "Minified files, saving {} bytes in {} seconds",
        old_size - dir_size("node_modules").unwrap(),
        start.elapsed().as_secs_f32()
    );

    let tar_gz = File::create("node_modules.tgz").unwrap();
    let enc = GzEncoder::new(&tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);
    tar.append_dir_all("node_modules", ".").unwrap();

    println!("{}", msg);
}

fn minify_js(path: String) {
    Command::new("cmd.exe")
        .arg("/C")
        .arg(format!(
            "esbuild.cmd {} --minify --allow-overwrite --outfile={}",
            path, path
        ))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .unwrap();
}
