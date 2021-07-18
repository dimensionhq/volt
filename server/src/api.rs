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

use std::io::Write;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VoltResponse {
    pub v: String,
    #[serde(flatten)]
    pub vs: HashMap<String, HashMap<String, VoltPackage>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VoltPackage {
    pub n: String,
    pub v: String,
    pub tb: String,
    pub s1: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub td: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub b: Option<HashMap<String, String>>,
    pub ig: String,
    #[serde(rename = "peerDependencies")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pd: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dp: Option<Vec<String>>,
}

impl VoltResponse {
    pub fn save(self, path: String) {
        let mut file = std::fs::File::create(path).unwrap();
        file.write_all(serde_json::to_string(&self).unwrap().as_bytes())
            .unwrap();
    }
}
