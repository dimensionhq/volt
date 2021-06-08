// MIT License
//
// Copyright (c) 2017 Guillaume Gomez
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

extern crate minifier;

use std::env;
use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use minifier::{css, js, json};

fn print_help() {
    println!(
        r##"For now, this minifier supports the following type of files:

 * .css
 * .js
 * .json"##
    );
}

pub fn get_all_data(file_path: &str) -> io::Result<String> {
    let mut file = File::open(file_path)?;
    let mut data = String::new();

    file.read_to_string(&mut data).unwrap();
    Ok(data)
}

fn call_minifier<F>(file_path: &str, func: F)
where
    F: Fn(&str) -> String,
{
    match get_all_data(file_path) {
        Ok(content) => {
            let mut out = PathBuf::from(file_path);
            let original_extension = out
                .extension()
                .unwrap_or_else(|| OsStr::new(""))
                .to_str()
                .unwrap_or("")
                .to_owned();
            out.set_extension(format!("min.{}", original_extension));
            if let Ok(mut file) = OpenOptions::new()
                .truncate(true)
                .write(true)
                .create(true)
                .open(out.clone())
            {
                if let Err(e) = write!(file, "{}", func(&content)) {
                    writeln!(
                        &mut io::stderr(),
                        "Impossible to write into {:?}: {}",
                        out,
                        e
                    )
                    .unwrap();
                } else {
                    println!("{:?}: done -> generated into {:?}", file_path, out);
                }
            } else {
                writeln!(
                    &mut io::stderr(),
                    "Impossible to create new file: {:?}",
                    out
                )
                .unwrap();
            }
        }
        Err(e) => writeln!(&mut io::stderr(), "\"{}\": {}", file_path, e).unwrap(),
    }
}

fn main() {
    let args: Vec<_> = env::args().skip(1).collect();

    if args.is_empty() {
        println!("Missing files to work on...\nExample: ./minifier file.js\n");
        print_help();
        return;
    }
    for arg in &args {
        let p = Path::new(arg);

        if !p.is_file() {
            writeln!(&mut io::stderr(), "\"{}\" isn't a file", arg).unwrap();
            continue;
        }
        match p
            .extension()
            .unwrap_or_else(|| OsStr::new(""))
            .to_str()
            .unwrap_or("")
        {
            "css" => call_minifier(arg, |s| css::minify(s).expect("css minification failed")),
            "js" => call_minifier(arg, js::minify),
            "json" => call_minifier(arg, json::minify),
            // "html" | "htm" => call_minifier(arg, html::minify),
            x => println!("\"{}\": this format isn't supported", x),
        }
    }
}
