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

use speedy::{Readable, Writable};
use std::collections::HashMap;

#[derive(Debug, Clone, Writable, Readable, Debug, Clone, Writable, Readable)]
struct VoltResponse {
    version: String,                    // the latest version of the package
    versions: Vec<String>,              // list of versions of the package
    tree: HashMap<String, VoltPackage>, // the flattened dependency tree for the latest version of the package <name@version, data>
}

#[derive(Debug, Clone, Writable, Readable)]
struct VoltPackage {
    pub name: String,                                       // the name of the package
    pub version: String,                                    // the version of the package
    pub integrity: String, // sha-1 base64 encoded hash or the "integrity" field if it exists
    pub tarball: String,   // url to the tarball to fetch
    pub bin: Option<HashMap<String, String>>, // binary scripts required by / for the package
    pub dependencies: Option<HashMap<String, String>>, // dependencies of the package
    pub dev_dependencies: Option<HashMap<String, String>>, // dev dependencies of the package
    pub peer_dependencies: Option<HashMap<String, String>>, // peer dependencies of the package
    pub peer_dependencies_meta: Option<HashMap<String, String>>, // peer dependencies metadata of the package
    pub optional_dependencies: Option<HashMap<String, String>>, // optional dependencies of the package
    pub overrides: Option<HashMap<String, String>>,             // overrides specific to the package
    pub engines: Option<HashMap<String, String>>, // engines compatible with the package
    pub os: Option<HashMap<String, String>>,      // operating systems compatible with the package
    pub cpu: Option<HashMap<String, String>>,     // cpu architectures compatible with the package
}