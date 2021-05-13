use std::{
    fs::File,
    io::{self, BufReader, BufWriter},
    path::PathBuf,
};
use thiserror::Error;

use serde::{Deserialize, Serialize};

#[derive(Error, Debug)]
pub enum LockFileError {
    #[error("unable to read lock file")]
    IO(io::Error),
    #[error("unable to deserialize lock file")]
    Decode(serde_json::Error),
    #[error("unable to serialize lock file")]
    Encode(serde_json::Error),
}

#[derive(Debug)]
pub struct LockFile {
    pub path: PathBuf,
    pub dependencies: Vec<DependencyLock>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DependencyLock {
    pub name: String,
    pub version: String,
    pub tarball: String,
    pub sha1: String,
}

impl LockFile {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            dependencies: Vec::new(),
        }
    }

    pub fn load(path: PathBuf) -> Result<Self, LockFileError> {
        let lock_file = File::open(&path).map_err(LockFileError::IO)?;
        let reader = BufReader::new(lock_file);

        Ok(LockFile {
            path,
            dependencies: serde_json::from_reader(reader).map_err(LockFileError::Decode)?,
        })
    }

    pub fn save(&self) -> Result<(), LockFileError> {
        let lock_file = File::create(&self.path).map_err(LockFileError::IO)?;
        let writer = BufWriter::new(lock_file);

        serde_json::to_writer_pretty(writer, &self.dependencies).map_err(LockFileError::Encode)
    }
}
