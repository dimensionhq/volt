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

use miette::Result;
use serde::{Deserialize, Serialize};
use speedy::{Readable, Writable};
use thiserror::Error;

use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufReader, Write},
    path::Path,
};

use crate::core::utils::voltapi::VoltPackage;

#[derive(Error, Debug)]
pub enum LockFileError {
    #[error("unable to read lock file")]
    IO(io::Error),
    #[error("unable to deserialize lock file")]
    #[allow(dead_code)]
    Decode(serde_json::Error),
    // #[error("unable to serialize lock file")]
    // Encode(serde_json::Error),
}

/// The lock file is responsible for locking/pinning dependency versions in a given project.
/// It stores a list of dependencies along with their resolved version, registry url, and sha1 checksum.
///
/// ## Examples
///
/// ```
/// // Load the lock file for the current project or create new lock file
/// let mut lock_file = LockFile::load(lock_file_path)
///     .unwrap_or_else(|| LockFile::new(lock_file_path));
///
/// // Add dependency
/// lock_file.add(
///     ("react".to_string(), "^1.0.0".to_string()),
///     DependencyLock {
///         name: "react".to_string(),
///         version: "1.2.6".to_string(),
///         tarball: String::new(),
///         sha1: String::new(),
///     }
/// );
///
/// // Save changes to disk
/// lock_file.save().expect("Unable to save lock file");
/// ```
#[derive(Clone, Debug, Writable, Readable, Serialize, Deserialize)]
pub struct LockFile {
    pub path: String,
    #[speedy(skip)]
    pub global: bool,
    #[speedy(skip)]
    pub dependencies: HashMap<String, VoltPackage>,
}

impl LockFile {
    /// Creates a new instance of a lock file with a path it should be saved at.
    /// It can be saved to the file by calling [`Self::save()`].
    pub fn new<P: AsRef<Path>>(path: P, global: bool) -> Self {
        Self {
            path: path.as_ref().to_str().unwrap().to_string(),
            global,
            dependencies: HashMap::with_capacity(1), // We will be installing at least 1 dependency
        }
    }

    pub fn add(package: VoltPackage) {}

    /// Loads a lock file from the given path.
    pub fn load<P: AsRef<Path>>(path: P, global: bool) -> Result<Self, LockFileError> {
        let path = path.as_ref();

        let dependencies = if path.exists() {
            let f = File::open(path).map_err(LockFileError::IO)?;
            let reader = BufReader::new(f);

            if global {
                LockFile::read_from_buffer(reader.buffer()).unwrap()
            } else {
                serde_json::from_reader(reader).unwrap()
            }
        } else {
            LockFile {
                path: path.to_str().unwrap().to_string(),
                global,
                dependencies: HashMap::new(),
            }
        };

        Ok(dependencies)
    }

    // Saves a lock file dumping pretty, formatted json
    // pub fn save_pretty(&self) -> Result<(), LockFileError> {
    //     let lock_file = File::create(&self.path).map_err(LockFileError::IO)?;
    //     let writer = BufWriter::new(lock_file);
    //     serde_json::to_writer_pretty(writer, &self.dependencies).map_err(LockFileError::Encode)
    // }

    // Saves a lock file to the same path it was opened from.
    pub fn save(&self) -> Result<()> {
        let mut lock_file = File::create(&self.path).unwrap();

        lock_file
            .write_all(&self.dependencies.write_to_vec().unwrap())
            .unwrap();

        Ok(())
    }
}
