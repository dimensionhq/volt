// Modules
mod minifier;

use std::time::Instant;

use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::{
    io::{Read, Write},
    path::Path,
};
use walkdir::WalkDir;

use flate2::write::GzEncoder;
use flate2::Compression;

enum SourceType {
    Css,
    Js,
    Json,
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
    let recognized_file_types = vec!["js", "ts", "jsx", "tsx", ".d.ts"];

    let ext = path.as_ref().extension()?;
    if ext == "css" {
        Some(SourceType::Css)
    } else if recognized_file_types.contains(&ext.to_str().unwrap()) {
        Some(SourceType::Js)
    } else if ext == "json" {
        Some(SourceType::Json)
    } else {
        Some(SourceType::Unknown)
    }
}

fn minify_file<P: AsRef<Path>>(
    path: P,
    src_ty: SourceType,
    buf: &mut String,
) -> std::io::Result<u64> {
    // Read
    buf.clear();

    // Open File
    File::open(&path)?.read_to_string(buf)?;

    let old_size = buf.len() as u64;

    let mut minified: String = String::new();

    // Minify
    match src_ty {
        SourceType::Css => {
            minified = minifier::json::minify(&buf);
        }
        SourceType::Json => {
            minified = minifier::json::minify(&buf);
        }
        SourceType::Js => {
            minified = minifier::js::minify(&buf);
        }
        SourceType::Unknown => {}
    }

    if minified != String::new() {
        let new_size = minified.len() as u64;

        if new_size >= old_size {
            return Ok(0);
        }

        // Write
        File::create(&path)?.write_all(minified.as_bytes())?;

        return Ok(old_size - new_size);
    } else {
        return Ok(0);
    }
}

fn main() {
    let start = Instant::now();
    let files = WalkDir::new("node_modules");

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

    let mut files_minified: u64 = 0;
    let mut space_saved = 0;

    let mut buf = String::new();
    for (entry, src_ty) in to_minify {
        let path = entry.path();

        let name = path.to_string_lossy().into_owned();
        pb.set_message(name);

        match minify_file(path, src_ty, &mut buf) {
            Ok(0) => {}
            Ok(n) => {
                files_minified += 1;
                space_saved += n;
            }
            Err(_e) => {}
        }

        // Either way, we're done with this file
        pb.inc(1);
    }

    pb.finish_and_clear();

    let msg = format!(
        "Minified {} files, saving {} bytes in {} seconds",
        files_minified,
        space_saved,
        start.elapsed().as_secs_f32()
    );

    let old_size = dir_size("node_modules").unwrap();
    println!("Minified Size: {}", old_size);

    let tar_gz = File::create("node_modules.tgz").unwrap();
    let enc = GzEncoder::new(&tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);
    tar.append_dir_all("node_modules", ".").unwrap();

    println!(
        "Minified + Compressed Size: {} bytes",
        tar_gz.metadata().unwrap().len()
    );

    println!("{}", msg);
}
