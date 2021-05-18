use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug)]
pub struct VoltResponse {
    pub version: String,
    #[serde(flatten)]  
    pub versions: HashMap<String, VersionData>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct VersionData {
    pub packages: HashMap<String, Package>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Package {    
    pub version: String,
    pub tarball: String,
    pub sha1: String,
    pub bin: Option<HashMap<String, String>>
}   