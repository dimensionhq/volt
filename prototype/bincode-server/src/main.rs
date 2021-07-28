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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BincodeVoltResponse {
    latest: String,
    schema: u8,
    versions: HashMap<String, HashMap<String, BincodeVoltPackage>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BincodeVoltPackage {
    sha1: Vec<u8>,
    sha512: Option<Vec<u8>>,
    dependencies: Option<Vec<String>>,
    peer_dependencies: Option<Vec<String>>,
}

fn main() {
    let files = read_dir("packages").unwrap();

    for file in files {
        let data = read_to_string(format!("{}", &file.as_ref().unwrap().path().display())).unwrap();

        let deserialized: VoltResponse = serde_json::from_str(&data).unwrap();
        let ds_clone = deserialized.clone();

        let mut versions: HashMap<String, HashMap<String, BincodeVoltPackage>> = HashMap::new();
        versions.insert(ds_clone.clone().latest, HashMap::new());

        let mut bincode_struct: BincodeVoltResponse = BincodeVoltResponse {
            latest: ds_clone.clone().latest,
            schema: ds_clone.clone().schema,
            versions: versions,
        };

        for (name, package) in deserialized
            .versions
            .get(&deserialized.latest)
            .unwrap()
            .iter()
        {
            let mut sha512: Option<Vec<u8>> = None;

            if package.integrity.contains("sha512-") {
                sha512 = Some(base64::decode(package.integrity.replace("sha512-", "")).unwrap());
            }

            let bincode_package: BincodeVoltPackage = BincodeVoltPackage {
                sha1: package
                    .sha1
                    .as_bytes()
                    .chunks(2)
                    .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
                    .collect(),
                sha512: sha512,
                dependencies: package.clone().dependencies,
                peer_dependencies: package.clone().peer_dependencies,
            };

            bincode_struct
                .versions
                .get_mut(&ds_clone.clone().latest)
                .unwrap()
                .insert(name.to_string(), bincode_package);
        }

        let file = File::create(format!(
            r"bin\{}.bin",
            file.unwrap()
                .path()
                .display()
                .to_string()
                .replace(".json", "")
                .replace("packages", "")
        ))
        .unwrap();

        let mut builder = EncoderBuilder::new();
        let mut encoder = builder.level(3).build(file).unwrap();

        bincode::serialize_into(&mut encoder, &bincode_struct).unwrap();

        let (_, _) = encoder.finish();

        // file.write_all(&data).unwrap();
    }
}
