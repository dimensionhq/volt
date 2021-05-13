use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct LockFile {
    pub freeze: Option<Vec<Lock>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Lock {
    pub name: String,
    pub version: String,
    pub tarball: String,
    pub sha1: String,
}

impl LockFile {
    pub fn new() -> Self {
        Self {
            freeze: Some(vec![]),
        }
    }
}
