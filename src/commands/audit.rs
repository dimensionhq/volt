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

//! Handle an unknown command (can be listed in scripts).

use crate::{App, Command};

use async_trait::async_trait;
use miette::Result;
use serde::{Deserialize, Serialize};

use std::{collections::HashMap, sync::Arc};

pub struct Audit {}

#[derive(Debug)]
pub struct AuditObject {
    name: String,
    version: String,
    install: Vec<String>,
    remove: Vec<String>,
    metadata: HashMap<String, String>,
    requires: HashMap<String, String>,
    dependencies: HashMap<String, AuditDependency>,
}

#[derive(Debug)]
pub struct AuditDependency {
    version: String,
    integrity: String,
    requires: HashMap<String, String>,
    dependencies: HashMap<String, AuditDependency>,
    dev: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditResponse {
    actions: Vec<String>,
    advisories: HashMap<String, String>,
    muted: Vec<String>,
    metadata: AuditMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Vulnerabilities {
    info: u128,
    low: u128,
    moderate: u128,
    high: u128,
    critical: u128,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditMetadata {
    vulnerabilities: Vulnerabilities,
    dependencies: u128,
    dev_dependencies: u128,
    optional_dependencies: u128,
    total_dependencies: u128,
}

// pub fn flatten_dependency_tree(
//     package: &VoltPackage,
//     volt_packages: &HashMap<String, VoltPackage>,
// ) -> Vec<AuditDependency> {
//     let mut packages: Vec<AuditDependency> = vec![];

//     if package.dependencies.is_some() {
//         for dep in package.dependencies.as_ref().unwrap().iter() {
//             let dependency = volt_packages.get(dep).unwrap();

//             let mut requires: HashMap<String, String> = HashMap::new();

//             if dependency.dependencies.is_some() {
//                 for de in dependency.dependencies.as_ref().unwrap() {
//                     let pkg = volt_packages.get(de).unwrap();
//                     requires.insert(de.to_owned(), pkg.version.clone());
//                 }
//             }

//             let fdt = &mut flatten_dependency_tree(dependency, volt_packages);

//             packages.push(AuditDependency {
//                 version: dependency.version,
//                 integrity: dependency.integrity,
//                 requires: requires,
//                 dependencies: HashMap::new()
//                     .insert(
//                         dep.to_owned(),
//                         ,
//                     )
//                     .unwrap(),
//                 dev: false,
//             });
//         }
//     }

//     packages
// }

#[async_trait]
impl Command for Audit {
    fn help() -> String {
        todo!()
    }

    /// Execute the `volt audit` command
    ///
    /// Execute a audit command
    /// ## Arguments
    /// * `error` - Instance of the command (`Arc<App>`)
    /// ## Examples
    /// ```
    /// //
    /// // .exec() is an async call so you need to await it
    /// Audit.exec(app).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(_app: Arc<App>) -> Result<()> {
        // let package_json = PackageJson::from("package.json");

        // let mut requires = package_json.dependencies;
        // requires.extend(package_json.dev_dependencies);

        // let responses =
        //     utils::get_volt_response_multi(requires.keys().cloned().collect::<Vec<String>>()).await;

        // let mut dependencies: HashMap<String, AuditDependency> = HashMap::new();

        // let start = Instant::now();

        // for res in responses {
        //     let version = res.version;

        //     let packages = &res.versions.get(&version).unwrap().packages;

        //     for package in packages {
        //         let flattened_dependency_tree: Vec<AuditDependency> =
        //             flatten_dependency_tree(package.1, packages);

        //         for dp in flattened_dependency_tree {
        //             dependencies.insert(package.0.to_owned(), dp);
        //         }
        //     }
        // }

        // let audit = AuditObject {
        //     name: package_json.name,
        //     version: package_json.version,
        //     install: vec![],
        //     remove: vec![],
        //     metadata: HashMap::new(),
        //     requires: requires,
        //     dependencies: dependencies,
        // };

        // let mut response = chttp::post_async(
        //     "http://registry.npmjs.org/-/npm/v1/security/audits",
        //     format!("{:?}", audit),
        // )
        // .await
        // .unwrap();

        // let text = response.text_async().await.unwrap();

        // let response: AuditResponse = serde_json::from_str(text.as_str()).unwrap();

        Ok(())
    }
}
