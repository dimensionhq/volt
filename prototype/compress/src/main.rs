use std::time::Instant;

// use indicatif::{ProgressBar, ProgressStyle};
// use minifier;
// use std::{
//     fs::File,
//     io::{Read, Write},
//     path::Path,
//     time::Instant,
// };
// use walkdir::WalkDir;

// enum SourceType {
//     Css,
//     Js,
//     Json,
// }

// fn minifiable<P: AsRef<Path>>(path: P) -> Option<SourceType> {
//     let ext = path.as_ref().extension()?;
//     if ext == "css" {
//         Some(SourceType::Css)
//     } else if ext == "js" {
//         Some(SourceType::Js)
//     } else if ext == "json" {
//         Some(SourceType::Json)
//     } else {
//         Some(SourceType::Js)
//     }
// }

// fn minify_file<P: AsRef<Path>>(
//     path: P,
//     src_ty: SourceType,
//     buf: &mut String,
// ) -> std::io::Result<u64> {
//     // Read
//     buf.clear();
//     File::open(&path)?.read_to_string(buf)?;

//     let old_size = buf.len() as u64;

//     // Minify
//     let minified = match src_ty {
//         SourceType::Css => minifier::css::minify(&buf).unwrap(),
//         SourceType::Json => minifier::json::minify(&buf),
//         SourceType::Js => minifier::js::minify(&buf),
//     };

//     let new_size = minified.len() as u64;

//     // Don't bother writing if the minification didn't help.
//     if new_size >= old_size {
//         return Ok(0);
//     }

//     // Write
//     File::create(&path)?.write_all(minified.as_bytes())?;

//     Ok(old_size - new_size)
// }

// fn main() {
//     let start = Instant::now();
//     let files = WalkDir::new("../../node_modules");

//     let to_minify: Vec<_> = files
//         .into_iter()
//         // Skip filesystem errors rather than panicking
//         .filter_map(Result::ok)
//         // Only look at files, not dirs or symlinks
//         .filter(|entry| entry.file_type().is_file())
//         // If something's minifiable, determine its type. If not, skip it.
//         .filter_map(|entry| {
//             let src_ty = minifiable(entry.file_name())?;
//             Some((entry, src_ty))
//         })
//         .collect();

//     // // Use this if it turns out the scanning phase takes a while.
//     // let mut to_minify = Vec::new();
//     // let pb = ProgressBar::new(0);
//     // pb.set_message("Scanning...");
//     // for entry in files.into_iter().filter_map(...).filter(...) {
//     //     if let Some(src_ty) = minifiable(entry.file_name()) {
//     //         to_minify.push((entry, src_ty));
//     //         pb.inc_len(1);
//     //     }
//     // }

//     let pb = ProgressBar::new(to_minify.len() as u64);
//     pb.set_style(
//         ProgressStyle::default_bar()
//             .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
//             .progress_chars("=> "),
//     );

//     let mut files_minified: u64 = 0;
//     let mut space_saved = 0;

//     let mut buf = String::new();
//     for (entry, src_ty) in to_minify {
//         let path = entry.path();

//         // if you want to say which thing you're minifying...
//         // this does involve allocation due to indicatif's API
//         // so if each file goes by fast enough, maybe don't do this
//         let name = path.to_string_lossy().into_owned();
//         pb.set_message(name);

//         match minify_file(path, src_ty, &mut buf) {
//             Ok(0) => {}
//             Ok(n) => {
//                 files_minified += 1;
//                 space_saved += n;
//             }
//             Err(_e) => {
//                 // Communicate the error?
//             }
//         }

//         // Either way, we're done with this file
//         pb.inc(1);
//     }

//     pb.finish_and_clear();

//     let msg = format!(
//         "Minified {} files, saving {} bytes in {} seconds",
//         files_minified,
//         space_saved,
//         start.elapsed().as_secs_f32()
//     );

//     println!("{}", msg);
// }

fn compress(provider: &str) {
    match provider {
        "zstd" => {}
        "tar" => {}
        "brotli" => {}
        "zip" => {}
        _ => {}
    }
}

fn main() {
    let start = Instant::now();
}
