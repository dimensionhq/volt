/*
    Copyright 2021 Volt Contributors

    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at

        http://www.apache.org/licenses/LICENSE-2.0

    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.
*/

use crate::{
    cli::VoltConfig,
    core::{classes::meta::Meta, utils::voltapi::VoltPackage},
};

use colored::Colorize;
use miette::IntoDiagnostic;
use ssri::Integrity;
use tar::Archive;

use std::{
    collections::HashMap,
    io::{Cursor, Read, Write},
    path::PathBuf,
};

pub fn write(text: &str, metadata: &Meta) {
    if !metadata.silent {
        if metadata.no_color {
            println!("{}", text.bright_white());
        } else {
            println!("{}", text);
        }
    }
}

pub fn write_verbose(text: &str, metadata: &Meta) {
    if !metadata.silent && metadata.verbose {
        if metadata.no_color {
            println!(
                "{}: {}",
                "verbose".bright_white().bold(),
                text.bright_white()
            );
        } else {
            println!(
                "{}: {}",
                "verbose".bright_green().bold(),
                text.bright_white()
            );
        }
    }
}

pub fn write_debug(text: &str, metadata: &Meta) {
    if !metadata.silent && metadata.debug {
        if metadata.no_color {
            println!("{}: {}", "debug".bright_white().bold(), text.bright_white());
        } else {
            println!(
                "{}: {}",
                "debug".bright_yellow().bold(),
                text.bright_white()
            );
        }
    }
}

pub fn extract_tarball(
    data: Vec<u8>,
    package: &VoltPackage,
    config: &VoltConfig,
) -> miette::Result<()> {
    // Generate the tarball archive given the decompressed bytes
    let mut node_archive = Archive::new(Cursor::new(data));

    // extract to both the global store + node_modules (in the case of them using the pnpm linking algorithm)
    let mut cas_file_map: HashMap<String, Integrity> = HashMap::new();

    // Add package's directory to list of created directories
    let mut created_directories: Vec<PathBuf> = vec![];

    for entry in node_archive.entries().into_diagnostic()? {
        let mut entry = entry.into_diagnostic()?;

        // Read the contents of the entry
        let mut buffer = Vec::with_capacity(entry.size() as usize);
        entry.read_to_end(&mut buffer).into_diagnostic()?;

        let entry_path_string = entry
            .path()
            .into_diagnostic()?
            .to_str()
            .expect("valid utf-8")
            .to_string();

        // Remove `package/` from `package/lib/index.js`
        let cleaned_entry_path_string =
            if let Some(i) = entry_path_string.char_indices().position(|(_, c)| c == '/') {
                &entry_path_string[i + 1..]
            } else {
                &entry_path_string[..]
            };

        // Create the path to the local .volt directory
        let mut package_directory = config.node_modules()?.join(VoltConfig::VOLT_HOME);

        // Add package's directory to it
        package_directory.push(package.directory_name());

        // push node_modules/.volt/send@0.17.2 to the list (because we created it in the previous step)
        created_directories.push(package_directory.clone());

        // Add the cleaned path to the package's directory
        let mut entry_path = package_directory.clone();

        entry_path.push("node_modules");

        entry_path.push(&package.name);

        entry_path.push(cleaned_entry_path_string);

        // Get the entry's parent
        let entry_path_parent = entry_path.parent().unwrap();

        // If we haven't created this directory yet, create it
        if !created_directories.iter().any(|p| p == entry_path_parent) {
            created_directories.push(entry_path_parent.to_path_buf());
            std::fs::create_dir_all(entry_path_parent).into_diagnostic()?;
        }

        let mut file_path = package_directory.join("node_modules");

        file_path.push(package.name.clone());

        file_path.push(cleaned_entry_path_string);

        // Write the contents to node_modules
        let mut file = std::fs::File::create(&file_path).unwrap();

        file.write_all(&buffer).into_diagnostic()?;

        // Write the contents of the entry into the content-addressable store located at `app.volt_dir`
        // We get a hash of the file
        let sri = cacache::write_hash_sync(&config.volt_home()?, &buffer).into_diagnostic()?;

        // Insert the name of the file and map it to the hash of the file
        cas_file_map.insert(entry_path_string, sri);
    }

    // Write the file, shasum map to the content-addressable store
    cacache::write_sync(
        &config.volt_home()?,
        &package.cacache_key(),
        serde_json::to_string(&cas_file_map).into_diagnostic()?,
    )
    .into_diagnostic()?;

    Ok(())
}
