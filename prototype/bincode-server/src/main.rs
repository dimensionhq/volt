use lz4::EncoderBuilder;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{read_dir, read_to_string, File},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VoltResponse {
    latest: String,
    schema: u8,
    versions: HashMap<String, HashMap<String, VoltPackage>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VoltPackage {
    sha1: String,
    integrity: String,
    peer_dependencies: Option<Vec<String>>,
    dependencies: Option<Vec<String>>,
}

fn main() {
    let files = read_dir("packages").unwrap();

    for file in files {
        let data = read_to_string(format!("{}", &file.as_ref().unwrap().path().display())).unwrap();

        let output_file = File::create(format!(
            r"compressed\{}.json",
            file.as_ref()
                .unwrap()
                .path()
                .display()
                .to_string()
                .replace(".json", "")
                .replace("packages", "")
        ))
        .unwrap();

        let mut builder = EncoderBuilder::new();
        let mut encoder = builder.level(5).build(output_file).unwrap();

        std::io::copy(
            &mut File::open(&file.as_ref().unwrap().path().display().to_string()).unwrap(),
            &mut encoder,
        )
        .unwrap();

        let (_, _) = encoder.finish();
    }
}
