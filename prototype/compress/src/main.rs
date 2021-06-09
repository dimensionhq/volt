use byte_unit::Byte;
use byte_unit::ByteUnit;
use std::fs::rename;
use std::os::windows::prelude::MetadataExt;
use std::time::Instant;

use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use walkdir::WalkDir;

use flate2::write::GzEncoder;
use flate2::Compression;

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
        .collect();

    let pb = ProgressBar::new(to_minify.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40.cyan/blue}]{pos:>7}/{len:7}files <> {msg}")
            .progress_chars("=> "),
    );

    let tar_gz = File::create("node_modules.tgz").unwrap();
    let enc = GzEncoder::new(&tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);
    tar.finish().unwrap();

    for entry in to_minify {
        let path = entry.path();

        let name = path.to_string_lossy().into_owned();
        let name_clone = name.clone().to_string();
        let file_name = name_clone.split(r"\").map(|x| x.to_owned()).last();

        pb.set_message(file_name.unwrap());

        tar.append_file(path, &mut File::open(path).unwrap())
            .unwrap();
        // Either way, we're done with this file
        pb.inc(1);
    }

    rename("node_modules.tgz", "node_modules.pack").unwrap();
    pb.finish_and_clear();

    let saved = old_size
        - File::open("node_modules.pack")
            .unwrap()
            .metadata()
            .unwrap()
            .file_size();

    let msg = format!(
        "Compressed files to node_modules.pack, saving {} in {:.2} seconds",
        Byte::from_unit(saved as f64, ByteUnit::B)
            .unwrap()
            .get_appropriate_unit(false)
            .to_string(),
        start.elapsed().as_secs_f32()
    );

    println!("{}", msg);
}
