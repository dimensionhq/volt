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

// Std Imports
use std::collections::HashMap;

// Library Imports
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Package {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "_rev")]
    pub rev: Option<String>,
    pub name: String,
    #[serde(rename = "dist-tags")]
    pub dist_tags: DistTags,
    pub versions: HashMap<String, Version>,
    // pub time: HashMap<String, String>,
    // pub maintainers: Vec<Maintainer>,
    // pub description: Option<String>,
    // pub homepage: Option<String>,
    // pub author: Option<Author>,
    // pub bugs: Option<Bugs>,
    // pub license: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct DistTags {
    pub latest: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Version {
    pub name: String,
    pub version: String,
    pub description: String,
    // pub author: Author,
    // pub license: String,
    // pub repository: Repository,
    // pub main: String,
    // pub module: String,
    // #[serde(rename = "jsnext:main")]
    // pub jsnext_main: String,
    // pub engines: Option<Engines>,
    // pub scripts: Scripts,
    pub dependencies: HashMap<String, String>,
    pub peer_dependencies: HashMap<String, String>,
    pub dev_dependencies: HashMap<String, String>,
    // pub git_head: String,
    // pub bugs: Bugs,
    // pub homepage: String,
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "_nodeVersion")]
    pub node_version: String,
    #[serde(rename = "_npmVersion")]
    pub npm_version: String,
    pub dist: Dist,
    // pub maintainers: Vec<Maintainer>,
    // #[serde(rename = "_npmUser")]
    // pub npm_user: NpmUser,
    // pub directories: Directories,
    // #[serde(rename = "_npmOperationalInternal")]
    // pub npm_operational_internal: NpmOperationalInternal,
    // #[serde(rename = "_hasShrinkwrap")]
    // pub has_shrinkwrap: bool,
}

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(default, rename_all = "camelCase")]
// pub struct Author {
//     pub name: String,
// }

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(default, rename_all = "camelCase")]
// pub struct Repository {
//     #[serde(rename = "type")]
//     pub type_field: String,
//     pub url: String,
// }

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Engines {
    pub node: String,
    pub npm: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Scripts {
    pub test: String,
    #[serde(rename = "test:watch")]
    pub test_watch: String,
    pub build: String,
    pub start: String,
    pub prepare: String,
    pub predeploy: String,
    pub deploy: String,
}

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(default, rename_all = "camelCase")]
// pub struct Bugs {
//     pub url: String,
// }

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Dist {
    pub integrity: String,
    pub shasum: String,
    pub tarball: String,
    pub file_count: i64,
    pub unpacked_size: i64,
    #[serde(rename = "npm-signature")]
    pub npm_signature: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Maintainer {
    pub name: String,
    pub email: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct NpmUser {
    pub name: String,
    pub email: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Directories {}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct NpmOperationalInternal {
    pub host: String,
    pub tmp: String,
}
