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

use super::errors::VoltError;

use miette::{IntoDiagnostic, Result};
use package_spec::PackageSpec;
use serde::{Deserialize, Serialize};

use std::{
    fs,
    io::Write,
    path::PathBuf,
    {collections::HashMap, fs::read_to_string},
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NpmPackage {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "_rev")]
    pub rev: Option<String>,
    pub name: String,
    #[serde(rename = "dist-tags")]
    pub dist_tags: HashMap<String, String>,
    pub versions: HashMap<String, Version>,
    pub time: HashMap<String, String>,
    pub maintainers: Vec<Maintainer>,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<Repository>,
    pub author: Option<Author>,
    pub keywords: Option<Vec<String>>,
    pub bugs: Option<Bugs>,
    pub license: Option<String>,
    pub readme: Option<String>,
}

// #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(default, rename_all = "camelCase")]
// pub struct DistTags {
//     pub latest: String,
//     pub stable: Option<String>,
//     pub canary: Option<String>,
//     pub dev: Option<String>,
//     pub beta: Option<String>,
//     pub alpha: Option<String>,
//     pub experimental: Option<String>,
// }

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Version {
    pub name: String,
    pub version: String,
    pub description: String,
    pub main: String,
    pub module: String,
    #[serde(rename = "jsnext:main")]
    pub jsnext_main: String,
    pub scripts: Scripts,
    pub dependencies: HashMap<String, String>,
    pub peer_dependencies: HashMap<String, String>,
    pub dev_dependencies: HashMap<String, String>,
    pub git_head: String,
    pub bugs: Bugs,
    pub homepage: String,
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "_nodeVersion")]
    pub node_version: String,
    #[serde(rename = "_npmVersion")]
    pub npm_version: String,
    pub dist: Dist,
    pub maintainers: Vec<Maintainer>,
    #[serde(rename = "_npmUser")]
    pub npm_user: NpmUser,
    pub directories: Directories,
    #[serde(rename = "_npmOperationalInternal")]
    pub npm_operational_internal: NpmOperationalInternal,
    #[serde(rename = "_hasShrinkwrap")]
    pub has_shrinkwrap: bool,
    pub readme: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Author {
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Repository {
    #[serde(rename = "type")]
    pub type_field: String,
    pub url: String,
}

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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Bugs {
    pub url: String,
}

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PackageJson {
    pub name: String,
    pub version: String,
    pub main: Option<String>,
    pub repository: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    #[serde(rename = "devDependencies")]
    #[serde(default)]
    pub dev_dependencies: HashMap<String, String>,
    #[serde(default)]
    pub scripts: HashMap<String, String>,
}
impl PackageJson {
    pub fn open(_path: &str) -> Result<(Self, PathBuf)> {
        for parent in std::env::current_dir()
            .map_err(|e| VoltError::EnvironmentError {
                env: String::from("CURRENT_DIR"),
                source: e,
            })?
            .ancestors()
        {
            let pkg_path = parent.join("package.json");

            if pkg_path.exists() {
                let data = read_to_string(&pkg_path).map_err(|e| VoltError::ReadFileError {
                    source: e,
                    name: pkg_path.to_str().unwrap().to_string(),
                })?;

                return Ok((
                    serde_json::from_str(data.as_str()).into_diagnostic()?,
                    pkg_path,
                ));
            }
        }

        miette::bail!("No package.json found!");
    }

    pub fn save(&self) -> Result<()> {
        let mut file = fs::File::create("package.json").into_diagnostic()?;

        file.write(
            serde_json::to_string_pretty(self)
                .into_diagnostic()?
                .as_bytes(),
        )
        .map_err(|e| VoltError::WriteFileError {
            source: e,
            name: String::from("package.json"),
        })?;

        Ok(())
    }

    pub fn add_dependency(&mut self, package: PackageSpec) {
        // self.dependencies
        //     .insert(package.name, package.version.unwrap_or_default());
    }

    // pub fn add_dev_dependency(&mut self, package: Package) {
    //     self.dev_dependencies
    //         .insert(package.name, package.version.unwrap_or_default());
    // }

    // pub fn remove_dev_dependency(&mut self, package: Package) {
    //     self.dev_dependencies.remove(&package.name);
    // }

    // pub fn remove_dependency(&mut self, package: Package) {
    //     self.dependencies.remove(&package.name);
    // }

    // pub fn update_dependency_version(
    //     &mut self,
    //     name: String,
    //     version: String,
    // ) -> Result<(), String> {
    //     if self.dependencies.contains_key(&name) {
    //         *self.dependencies.get_mut(&name).unwrap() = version.to_string();
    //         Ok(())
    //     } else {
    //         Err(String::from("dependency does not exist on the hashmap"))
    //     }
    // }
}
