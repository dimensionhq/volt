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

//! Check for outdated packages.
use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use clap::{ArgMatches, Parser, Subcommand};
use colored::Colorize;
use miette::Result;
use node_semver::{Range, Version};
use serde::{Deserialize, Deserializer};
use serde_json;

use crate::{
    cli::{VoltCommand, VoltConfig},
    core::utils::extensions::PathExtensions,
    core::utils::package::PackageJson,
};

// https://github.com/npm/registry/blob/master/docs/REGISTRY-API.md#version
// abbreviated version uses "Accept: application/vnd.npm.install-v1+json" in header
// this struct is specifically for the appreviated package responses from the registry.
// this method is used to ensure faster responses when doing all dependencies.
// all response fields must be optional in order to allow for different response
// types, and the error response, which only contains a single "error" field.
#[derive(Deserialize, Debug)]
pub struct PackageResponse {
    // resource for handling dynamic registry responses with Rust
    // https://hamatti.org/posts/learning-rust-4-parsing-json-with-strong-types/

    // resource for serde aliasing since the registry response uses "dist-tags"
    // https://stackoverflow.com/questions/71328963/how-to-read-response-json-as-structs-when-they-contain-hyphens-in-key-names
    name: Option<String>, // string name of the package
    #[serde(alias = "dist-tags")]
    dist_tags: Option<HashMap<String, String>>, // will always contain at least one tag which is "latest", for latest version
    versions: Option<HashMap<String, PackageVersion>>, // dictionary of versions with key: semver version string, value: PackageVersion struct
    modified: Option<String>,                          // string date of package last modified date
    error: Option<String>,                             // registry request string error
}

#[derive(Deserialize, Debug)]
struct PackageVersion {
    name: String,    // package name
    version: String, // semver version string
}

#[derive(Debug, Parser)]
pub struct Outdated {
    package: Option<String>,
}

#[async_trait]
impl VoltCommand for Outdated {
    // TODO: figure out how to run with & without arguments

    // TECHNICALLY DONE.
    // TODO: figure out how to combine with primary dependencies, to check each dependency/version
    //       on a project.

    // TECHNICALLY DONE
    // TODO: understand how semver works and can be used to our advantage

    // TODO: validate that the user provided a dependency/package that is actually installed before
    //       sending a request to the registry.

    // TECHNICALLY DONE, SHOULD ONLY ACCEPT ONE VERSION
    // TODO: Need to handle version ranges and exact versions separately

    async fn exec(self, _: VoltConfig) -> Result<()> {
        // realistically the 'node_modules' file should be in the same directory
        // as the primary package.json file for a project.
        let base_url = "https://registry.npmjs.org/";

        // TODO:
        // make this result optional entirely for when 'package.json' file doesn't exist!
        let primary_pkg = PackageJson::get().unwrap();
        let mut primary_deps = primary_pkg.0.dependencies;
        let mut node_modules = primary_pkg.1;
        node_modules.pop();
        node_modules.push("node_modules");

        //println!("Node modules are at {:?}", node_modules);

        // add to the modules path for single dependency, so we can have the
        // path to that dependency's folder within 'node_modules'
        //modules.push(&self.dependency);

        //println!("Dependency is at {modules:?}");

        // grab the 'package.json' file for this specific dependency in order
        // to get exact version information.
        //let pkg_json = PackageJson::get_from_dir(modules).unwrap().0;
        // convert the semver version string from the PackageJson struct into
        // a node_semver Version struct which can be used to compare to the
        // semver version string 'latest' returned in the registry response.
        //let current: Version = pkg_json.version.parse().unwrap();
        //println!("Got version {}", pkg_json.version);
        //let deps = pkgjson.dependencies.unwrap();

        //let pwd = std::env::current_dir().unwrap();
        //println!("Running from {pwd:?}");

        // in this case there are no packages installed in a project!
        if let Some(deps) = &primary_deps {
            //if !primary_deps.is_none() {
            // test to see if an argument was provided, if not run for all dependencies???
            if let Some(package_name) = self.package {
                //&& !self.dependency.contains("!") {
                if (!deps.contains_key(&package_name)) {
                    let output = format!(
                        "{} is not an installed package!",
                        &package_name.truecolor(255, 000, 000)
                    );
                    println!("{}", output);
                } else {
                    //println!("Arguments: {}\n", self.dependency);

                    let mut single = node_modules.clone();
                    single.push(&package_name);

                    //let pkg_json: PackageJson = PackageJson::get_from_dir(&single).unwrap().0;
                    //let current: Version = pkg_json.version.parse().unwrap();

                    // println!("Dependency is at {single:?}");
                    //pkg_json = PackageJson::get_from_dir(&single).unwrap().0;
                    //current = pkg_json.version.parse().unwrap();
                    //pkg_json = PackageJson::get_from_dir(&single).unwrap().0;
                    //current = pkg_json.version.parse().unwrap();
                    //}
                    // if !deps.contains_key(&self.dependency) {
                    //     println!("Dependency not installed!");
                    //     std::process::exit(1);
                    // }
                    // let curr = &deps[&self.dependency];

                    // println!("current version is {curr}");
                    // let current: Range = deps[&self.dependency].parse().unwrap();
                    // need client to add headers
                    let client = reqwest::Client::new();

                    // NOTE: biggest help for handling dynamic JSON responses was hamatti.org!!!!
                    // https://hamatti.org/posts/learning-rust-4-parsing-json-with-strong-types/
                    // https://stackoverflow.com/questions/47911513/how-do-i-set-the-request-headers-using-reqwest
                    // https://stackoverflow.com/questions/63872942/how-can-an-arbitrary-json-structure-be-deserialized-with-reqwest-get-in-rust
                    // https://stackoverflow.com/questions/68357867/dynamically-receive-json-data-in-rust-with-reqwest

                    /* working version without serde_json structure, this format uses assumed structure...
                    let package_info = client
                        .get(format!("{}/{}", base_url, &self.dependency))
                        .header("Accept", "application/vnd.npm.install-v1+json")
                        .send()
                        .await
                        .unwrap()
                        .json::<serde_json::Value>()
                        .await
                        .unwrap();
                    println!("Testing: {}", package_info);
                    */

                    // this format assigns the JSON into the appropriate
                    // fields within the Package struct.
                    let package_info: PackageResponse = client
                        .get(format!("{}/{}", base_url, &package_name))
                        .header("Accept", "application/vnd.npm.install-v1+json")
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();

                    // check to see if the optional error value was none
                    // which indicates the request was successful.
                    if package_info.error.is_none() {
                        //println!("Package name: {}\n", package_info.name.unwrap());
                        //println!("Tags: {:?}", package_info.dist_tags.as_ref().unwrap());

                        let latest: Version = package_info
                            .dist_tags
                            .as_ref()
                            .unwrap()
                            .get("latest")
                            .unwrap()
                            .parse()
                            .unwrap();

                        println!("Package \t\t| Current \t\t| Latest\n---------------------------------------------------------");
                        if !single.exists() {
                            let output = format!(
                                "{} \t\t| {} \t\t| {}",
                                package_info.name.unwrap().truecolor(255, 000, 000),
                                "MISSING",
                                latest.to_string().truecolor(055, 125, 235),
                            );
                            println!("{}", output);
                        } else {
                            let pkg_json: PackageJson =
                                PackageJson::get_from_dir(&single).unwrap().0;
                            let current: Version = pkg_json.version.parse().unwrap();
                            if current < latest {
                                let output = format!(
                                    "{} \t\t| {} \t\t| {}",
                                    package_info.name.unwrap().truecolor(255, 000, 000),
                                    current,
                                    latest.to_string().truecolor(055, 125, 235),
                                );
                                println!("{}", output);
                            } else {
                                let output = format!(
                                    "{} \t\t| {} \t\t| {}",
                                    package_info.name.unwrap().truecolor(000, 255, 000),
                                    current,
                                    latest.to_string().truecolor(055, 125, 235),
                                );
                                println!("{}", output);
                            }
                        }
                    } else {
                        let output = format!(
                            "Error fetching package info for {} due to: \n\t{}",
                            package_info.name.unwrap(),
                            package_info.error.unwrap().truecolor(255, 000, 000)
                        );
                        println!("{}", output);
                    }
                }
            } else {
                //println!("No dependency provided, run for all????");

                let mut multiple = node_modules.clone();
                //println!("{:?}", &multiple);

                let mut found_outdated = false;

                let dependencies = primary_deps.unwrap();

                for dependency in dependencies {
                    //multiple.push(dependency.0);
                    //println!("Dependency is at {multiple:?}");
                    let dep_name = dependency.0;
                    //println!("{:?}", &multiple.join(&dep_name));

                    let client = reqwest::Client::new();

                    let package_info: PackageResponse = client
                        .get(format!("{}/{}", base_url, &dep_name))
                        .header("Accept", "application/vnd.npm.install-v1+json")
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();

                    // check to see if the optional error value was none
                    // which indicates the request was successful.
                    if package_info.error.is_none() {
                        //println!("Package name: {}\n", package_info.name.unwrap());
                        //println!("Tags: {:?}", package_info.dist_tags.as_ref().unwrap());

                        let latest: Version = package_info
                            .dist_tags
                            .as_ref()
                            .unwrap()
                            .get("latest")
                            .unwrap()
                            .parse()
                            .unwrap();

                        //println!("Latest version: {}", latest); :? is debug print
                        //println!("Current version: {:?}", current);

                        if !multiple.join(&dep_name).exists() {
                            if (!found_outdated) {
                                println!("Package \t\t| Current \t\t| Latest\n---------------------------------------------------------");
                                found_outdated = true;
                            }

                            let output = format!(
                                "{} \t\t| {} \t\t| {}",
                                package_info.name.unwrap().truecolor(255, 000, 000),
                                "MISSING",
                                latest.to_string().truecolor(55, 125, 235),
                            );
                            println!("{}", output);
                        } else {
                            let pkg_json = PackageJson::get_from_dir(&multiple.join(&dep_name))
                                .unwrap()
                                .0;
                            let current: Version = pkg_json.version.parse().unwrap();
                            if current < latest {
                                if (!found_outdated) {
                                    println!("Package \t\t| Current \t\t| Latest\n---------------------------------------------------------");
                                    found_outdated = true;
                                }

                                let output = format!(
                                    "{} \t\t| {} \t\t| {}",
                                    package_info.name.unwrap().truecolor(255, 000, 000),
                                    current,
                                    latest.to_string().truecolor(55, 125, 235),
                                );
                                println!("{}", output);
                            }
                        }
                    } else {
                        let output = format!(
                            "Error fetching package info for {} due to: \n\t{}",
                            package_info.name.unwrap(),
                            package_info.error.unwrap().truecolor(255, 000, 000)
                        );
                        println!("{}", output);
                    }
                }

                if (!found_outdated) {
                    println!("All packages are up to date!")
                }
            }
        } else {
            println!("No packages are installed in the current project!")
        }
        Ok(())
    }
}
