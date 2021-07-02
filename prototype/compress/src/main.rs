use std::process::Command;

use jwalk::WalkDir;

fn minify() {}

fn main() {

    let mut idx = 0;
    for entry in WalkDir::new("node_modules") {
        let entry = entry.unwrap();

        if entry.path().is_file() {
            if entry
                .path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .ends_with(".js")
            {
                
                idx += 1;
            }
        }
    }
}
