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

#[derive(Debug, Clone, PartialEq, Writable, Readable)]
pub struct VoltResponse {
    pub version: String,
    pub versions: HashMap<String, VoltPackage>,
}

#[derive(Debug, Clone, PartialEq, Writable, Readable)]
pub struct VoltPackage {
    pub name: String,
    pub version: String,
    pub tarball: String,
    pub bin: Option<HashMap<String, String>>,
    pub integrity: String,
    pub peer_dependencies: Option<Vec<String>>,
    pub dependencies: Option<Vec<String>>,
}

#[derive(Debug, Clone, Writable, Readable)]
pub struct SpeedyVoltResponse {
    pub version: String,
    pub versions: HashMap<String, SpeedyVoltPackage>,
}

#[derive(Debug, Clone, Writable, Readable)]
pub struct SpeedyVoltPackage {
    pub integrity: String,
    pub tarball: String,
    pub bin: Option<HashMap<String, String>>,
    pub dependencies: Option<Vec<String>>,
    pub peer_dependencies: Option<Vec<String>>,
}
