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

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct VoltResponse {
    pub version: String,
    #[serde(flatten)]
    pub versions: HashMap<String, VoltPackage>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct VoltPackage {
    pub name: String,
    pub version: String,
    pub tarball: String,
    pub bin: Option<HashMap<String, String>>,
    pub integrity: String,
    pub peer_dependencies: Option<Vec<String>>,
    pub dependencies: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JSONVoltResponse {
    #[serde(flatten)]
    pub versions: HashMap<String, JSONVoltPackage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JSONVoltPackage {
    pub integrity: String,
    pub tarball: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bin: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_dependencies: Option<Vec<String>>,
}
