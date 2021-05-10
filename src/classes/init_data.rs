use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;

#[derive(Serialize, Deserialize)]
pub struct InitData {
    pub name: String,
    pub version: String,
    pub description: String,
    pub main: String,
    pub repository: String,
    pub author: String,
    pub license: String,
    pub private: bool,
}

impl InitData {
    // pub fn load(&self) -> InitData {

    // }

    pub fn dump(&self) -> String {
        to_string_pretty(&self).unwrap()
    }
}
