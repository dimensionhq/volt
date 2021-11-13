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

use crate::commands::add::PackageInfo;
use crate::core::utils::{constants::MAX_RETRIES, errors::VoltError, voltapi::VoltPackage};

use colored::Colorize;
use futures::{stream::FuturesOrdered, TryStreamExt};
use indicatif::ProgressBar;
use isahc::{
    config::{Configurable, SslOption},
    http::StatusCode,
    AsyncReadResponseExt, Request, RequestExt,
};
use miette::Result;
use semver_rs::Version;
use serde_json::Value;
use ssri::{Algorithm, Integrity};
use std::time::Instant;

pub fn parse_versions(packages: &[String]) -> Result<Vec<PackageInfo>> {
    let mut parsed: Vec<PackageInfo> = vec![];

    for package in packages.iter() {
        let split = package.split('@').map(|s| s.trim()).collect::<Vec<&str>>();
        let length = split.len();

        if length == 1 {
            parsed.push(PackageInfo {
                name: split[0].to_string(),
                version: None,
            });
        } else if length == 2 && !package.contains('/') {
            parsed.push(PackageInfo {
                name: split[0].to_string(),
                version: Some(split[1].to_string()),
            });
        } else if length == 2 && package.contains('/') {
            parsed.push(PackageInfo {
                name: format!("@{}", split[1]),
                version: None,
            });
        } else if length == 3 && package.contains('/') {
            parsed.push(PackageInfo {
                name: format!("@{}", split[1]),
                version: Some(split[2].to_string()),
            });
        }
    }

    Ok(parsed)
}

// Get version from NPM
pub async fn get_version(
    package_info: PackageInfo,
) -> Result<(PackageInfo, String, VoltPackage, bool)> {
    let mut retries = 0;

    let package_name = package_info.name;

    let count = package_name.matches('@').count();

    if (count == 1 && package_name.contains('/')) || (count == 0 && !package_name.contains('/')) {
        loop {
            let client: Request<&str> =
                Request::get(format!("https://registry.npmjs.org/{}", package_name))
                    .header(
                        "accept",
                        "application/vnd.npm.install-v1+json; q=1.0, application/json; q=0.8, */*",
                    )
                    .header("accept-encoding", "gzip,deflate")
                    .header("connection", "keep-alive")
                    .header("host", "registry.npmjs.org")
                    .ssl_options(SslOption::DANGER_ACCEPT_REVOKED_CERTS)
                    .body("")
                    .map_err(VoltError::RequestBuilderError)?;

            let mut response = client.send_async().await.unwrap_or_else(|e| {
                println!("{:?}", e);
                std::process::exit(1);
            });

            match *response.status_mut() {
                StatusCode::OK => {
                    let text = response.text().await.map_err(VoltError::IoTextRecError)?;

                    let json = serde_json::from_str::<Value>(&text).unwrap();

                    match json["dist-tags"]["latest"].as_str() {
                        Some(latest) => {
                            let num_deps;

                            match json["versions"][latest]["dependencies"].as_object() {
                                Some(value) => {
                                    num_deps = value.keys().count();
                                }
                                None => {
                                    num_deps = 0;
                                }
                            }

                            let package: VoltPackage;

                            match json["versions"][latest]["dist"].as_object() {
                                Some(value) => {
                                    let hash_string: String;

                                    if value.contains_key("integrity") {
                                        hash_string =
                                            value["integrity"].to_string().replace("\"", "");
                                    } else {
                                        hash_string = format!(
                                            "sha1-{}",
                                            base64::encode(value["shasum"].to_string())
                                        );
                                    }

                                    let integrity: Integrity =
                                        hash_string.parse().map_err(|_| {
                                            VoltError::HashParseError {
                                                hash: hash_string.to_string(),
                                            }
                                        })?;

                                    let algo = integrity.pick_algorithm();

                                    let mut hash = integrity
                                        .hashes
                                        .into_iter()
                                        .find(|h| h.algorithm == algo)
                                        .map(|h| Integrity { hashes: vec![h] })
                                        .map(|i| i.to_hex().1)
                                        .ok_or(VoltError::IntegrityConversionError)?;

                                    match algo {
                                        Algorithm::Sha1 => {
                                            hash = format!("sha1-{}", hash);
                                        }
                                        Algorithm::Sha512 => {
                                            hash = format!("sha512-{}", hash);
                                        }
                                        _ => {}
                                    }

                                    package = VoltPackage {
                                        name: package_name.clone(),
                                        version: latest.to_string(),
                                        tarball: value["tarball"].to_string().replace("\"", ""),
                                        bin: None,
                                        integrity: hash.clone(),
                                        peer_dependencies: None,
                                        dependencies: None,
                                    };

                                    return Ok((
                                        PackageInfo {
                                            name: package_name,
                                            version: Some(latest.to_string()),
                                        },
                                        hash,
                                        package,
                                        num_deps == 0,
                                    ));
                                }
                                None => {
                                    return Err(VoltError::HashLookupError {
                                        version: latest.to_string(),
                                    }
                                    .into());
                                }
                            }
                        }
                        None => {
                            return Err(VoltError::VersionLookupError { name: package_name }.into());
                        }
                    }
                }
                StatusCode::NOT_FOUND => {
                    if retries == MAX_RETRIES {
                        return Err(VoltError::TooManyRequests {
                            url: format!("https://registry.npmjs.org/{}", package_name),
                            package_name: package_name.to_string(),
                        }
                        .into());
                    }
                }
                _ => {
                    if retries == MAX_RETRIES {
                        return Err(VoltError::PackageNotFound {
                            url: format!("https://registry.npmjs.org/{}", package_name),
                            package_name: package_name.to_string(),
                        }
                        .into());
                    }
                }
            }

            retries += 1;
        }
    } else if count == 2 && package_name.contains('/') {
        let input_version = package_name.split('@').collect::<Vec<&str>>()[2].to_string();

        let version_requirement = semver_rs::Range::new(&input_version).parse().unwrap();

        loop {
            let name = format!("@{}", input_version);

            let client: Request<&str> = Request::get(format!(
                "https://registry.npmjs.org/{}",
                package_name.replace(&name, "")
            ))
            .header(
                "accept",
                "application/vnd.npm.install-v1+json; q=1.0, application/json; q=0.8, */*",
            )
            .header("accept-encoding", "gzip, deflate, br")
            .header("connection", "keep-alive")
            .header("host", "registry.npmjs.org")
            .body("")
            .map_err(VoltError::RequestBuilderError)?;

            let mut response = client.send_async().await.unwrap_or_else(|e| {
                println!("{}", e);
                std::process::exit(1);
            });

            match *response.status_mut() {
                StatusCode::OK => {
                    let text = response.text().await.map_err(VoltError::IoTextRecError)?;

                    let json = serde_json::from_str::<Value>(&text).unwrap();

                    match json["versions"].as_object() {
                        Some(value) => {
                            let mut available_versions = value
                                .keys()
                                .filter_map(|k| Version::new(k).parse().ok())
                                .filter(|v| version_requirement.test(v))
                                .collect::<Vec<_>>();

                            available_versions
                                .sort_unstable_by(|a, b| a.partial_cmp(b).unwrap().reverse());

                            if available_versions.is_empty() {
                                return Err(
                                    VoltError::VersionLookupError { name: package_name }.into()
                                );
                            }

                            let num_deps;

                            match json["versions"][available_versions[0].to_string()]
                                ["dependencies"]
                                .as_object()
                            {
                                Some(value) => {
                                    num_deps = value.keys().count();
                                }
                                None => {
                                    num_deps = 0;
                                }
                            }

                            let package: VoltPackage;

                            match json["versions"][available_versions[0].to_string()]["dist"]
                                .as_object()
                            {
                                Some(value) => {
                                    let hash_string: String;

                                    if value.contains_key("integrity") {
                                        hash_string =
                                            value["integrity"].to_string().replace("\"", "");
                                    } else {
                                        hash_string = format!(
                                            "sha1-{}",
                                            base64::encode(value["shasum"].to_string())
                                        );
                                    }

                                    let integrity: Integrity =
                                        hash_string.parse().map_err(|_| {
                                            VoltError::HashParseError {
                                                hash: hash_string.to_string(),
                                            }
                                        })?;

                                    let algo = integrity.pick_algorithm();

                                    let mut hash = integrity
                                        .hashes
                                        .into_iter()
                                        .find(|h| h.algorithm == algo)
                                        .map(|h| Integrity { hashes: vec![h] })
                                        .map(|i| i.to_hex().1)
                                        .ok_or(VoltError::IntegrityConversionError)?;

                                    match algo {
                                        Algorithm::Sha1 => {
                                            hash = format!("sha1-{}", hash);
                                        }
                                        Algorithm::Sha512 => {
                                            hash = format!("sha512-{}", hash);
                                        }
                                        _ => {}
                                    }

                                    package = VoltPackage {
                                        name: package_name.replace(&name, ""),
                                        version: input_version,
                                        tarball: value["tarball"].to_string().replace("\"", ""),
                                        bin: None,
                                        integrity: hash.clone(),
                                        peer_dependencies: None,
                                        dependencies: None,
                                    };

                                    return Ok((
                                        PackageInfo {
                                            name: package_name,
                                            version: Some(available_versions[0].to_string()),
                                        },
                                        hash,
                                        package,
                                        num_deps == 0,
                                    ));
                                }
                                None => {
                                    return Err(VoltError::HashLookupError {
                                        version: available_versions[0].to_string(),
                                    }
                                    .into());
                                }
                            }
                        }
                        None => {
                            return Err(VoltError::VersionLookupError { name: package_name }.into());
                        }
                    }
                }
                StatusCode::NOT_FOUND => {
                    if retries == MAX_RETRIES {
                        return Err(VoltError::TooManyRequests {
                            url: format!("https://registry.npmjs.org/{}", package_name),
                            package_name: package_name.to_string(),
                        }
                        .into());
                    }
                }
                _ => {
                    return Err(VoltError::PackageNotFound {
                        url: format!("https://registry.npmjs.org/{}", package_name),
                        package_name: package_name.to_string(),
                    }
                    .into());
                }
            }

            retries += 1;
        }
    } else if count == 1 && !package_name.contains('/') {
        let input_version = package_name.split('@').collect::<Vec<&str>>()[1].to_string();

        let version_requirement = semver_rs::Range::new(&input_version).parse().unwrap();

        loop {
            let name = format!("@{}", input_version);

            let client: Request<&str> = Request::get(format!(
                "https://registry.npmjs.org/{}",
                package_name.replace(&name, "")
            ))
            .header(
                "accept",
                "application/vnd.npm.install-v1+json; q=1.0, application/json; q=0.8, */*",
            )
            .header("accept-encoding", "gzip, deflate, br")
            .header("connection", "keep-alive")
            .header("host", "registry.npmjs.org")
            .body("")
            .map_err(VoltError::RequestBuilderError)?;

            let mut response = client.send_async().await.unwrap_or_else(|e| {
                eprintln!("{:?}", e);
                std::process::exit(1);
            });

            match *response.status_mut() {
                StatusCode::OK => {
                    let text = response.text().await.map_err(VoltError::IoTextRecError)?;

                    let json = serde_json::from_str::<Value>(&text).unwrap();

                    if let Some(value) = json["versions"].as_object() {
                        let mut available_versions = value
                            .keys()
                            .filter_map(|k| Version::new(k).parse().ok())
                            .filter(|v| version_requirement.test(v))
                            .collect::<Vec<_>>();

                        available_versions
                            .sort_unstable_by(|a, b| a.partial_cmp(b).unwrap().reverse());

                        if available_versions.is_empty() {
                            return Err(VoltError::VersionLookupError { name: package_name }.into());
                        }

                        let num_deps;

                        match json["versions"][available_versions[0].to_string()]["dependencies"]
                            .as_object()
                        {
                            Some(value) => {
                                num_deps = value.keys().count();
                            }
                            None => {
                                num_deps = 0;
                            }
                        }

                        let package: VoltPackage;

                        match json["versions"][available_versions[0].to_string()]["dist"]
                            .as_object()
                        {
                            Some(value) => {
                                let hash_string: String;

                                if value.contains_key("integrity") {
                                    hash_string = value["integrity"].to_string().replace("\"", "");
                                } else {
                                    hash_string = format!(
                                        "sha1-{}",
                                        base64::encode(value["shasum"].to_string())
                                    );
                                }

                                let integrity: Integrity =
                                    hash_string.parse().map_err(|_| VoltError::HashParseError {
                                        hash: hash_string.to_string(),
                                    })?;

                                let algo = integrity.pick_algorithm();

                                let mut hash = integrity
                                    .hashes
                                    .into_iter()
                                    .find(|h| h.algorithm == algo)
                                    .map(|h| Integrity { hashes: vec![h] })
                                    .map(|i| i.to_hex().1)
                                    .ok_or(VoltError::IntegrityConversionError)?;

                                match algo {
                                    Algorithm::Sha1 => {
                                        hash = format!("sha1-{}", hash);
                                    }
                                    Algorithm::Sha512 => {
                                        hash = format!("sha512-{}", hash);
                                    }
                                    _ => {}
                                }

                                package = VoltPackage {
                                    name: package_name.replace(&name, ""),
                                    version: input_version,
                                    tarball: value["tarball"].to_string().replace("\"", ""),
                                    bin: None,
                                    integrity: hash.clone(),
                                    peer_dependencies: None,
                                    dependencies: None,
                                };

                                return Ok((
                                    PackageInfo {
                                        name: package_name,
                                        version: Some(available_versions[0].to_string()),
                                    },
                                    hash,
                                    package,
                                    num_deps == 0,
                                ));
                            }
                            None => {
                                return Err(VoltError::HashLookupError {
                                    version: available_versions[0].to_string(),
                                }
                                .into());
                            }
                        }
                    }
                }
                StatusCode::NOT_FOUND => {
                    if retries == MAX_RETRIES {
                        return Err(VoltError::VersionLookupError { name: package_name }.into());
                    }
                }
                _ => {
                    if retries == MAX_RETRIES {
                        return Err(VoltError::PackageNotFound {
                            url: format!("https://registry.npmjs.org/{}", package_name),
                            package_name: package_name.to_string(),
                        }
                        .into());
                    }
                }
            }

            retries += 1;
        }
    } else {
        Err(VoltError::UnknownError.into())
    }
}

pub async fn get_versions(
    packages: &[PackageInfo],
    bar: &ProgressBar,
) -> Result<Vec<(PackageInfo, String, VoltPackage, bool)>> {
    packages
        .to_owned()
        .into_iter()
        .map(|v| {
            bar.set_message(format!("{}:{}", "npm".bright_magenta().bold(), v.name));
            get_version(v)
        })
        .collect::<FuturesOrdered<_>>()
        .try_collect::<Vec<(PackageInfo, String, VoltPackage, bool)>>()
        .await
}
