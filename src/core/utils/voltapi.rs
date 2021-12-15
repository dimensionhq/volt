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
use speedy::{Readable, Writable};
use std::collections::HashMap;

#[derive(Debug, Clone, Writable, Readable)]
pub struct VoltResponse {
    pub version: String,                    // the latest version of the package
    pub versions: Vec<String>,              // list of versions of the package
    pub tree: HashMap<String, VoltPackage>, // the flattened dependency tree for the latest version of the package <name@version, data>
}

#[derive(Debug, Clone, Writable, Readable)]
pub struct VoltPackage {
    pub name: String,                                       // the name of the package
    pub version: String,                                    // the version of the package
    pub integrity: String, // sha-1 base64 encoded hash or the "integrity" field if it exists
    pub tarball: String,   // url to the tarball to fetch
    pub bin: Option<Bin>,  // binary scripts required by / for the package
    pub scripts: Option<HashMap<String, String>>, // scripts required by / for the package
    pub dependencies: Option<HashMap<String, String>>, // dependencies of the package
    pub peer_dependencies: Option<HashMap<String, String>>, // peer dependencies of the package
    pub peer_dependencies_meta: Option<HashMap<String, String>>, // peer dependencies metadata of the package
    pub optional_dependencies: Option<HashMap<String, String>>, // optional dependencies of the package
    pub overrides: Option<HashMap<String, String>>,             // overrides specific to the package
    pub engines: Option<Engine>,  // engines compatible with the package
    pub os: Option<Vec<String>>,  // operating systems compatible with the package
    pub cpu: Option<Vec<String>>, // cpu architectures compatible with the package
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Readable, Writable)]
#[serde(untagged)]
pub enum Engine {
    String(String),
    List(Vec<String>),
    Map(HashMap<String, String>),
}

impl Default for Engine {
    fn default() -> Self {
        Engine::String(String::new())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Readable, Writable)]
#[serde(untagged)]
pub enum Bin {
    String(String),
    Map(HashMap<String, String>),
}

impl Default for Bin {
    fn default() -> Self {
        Bin::String(String::new())
    }
}
