use crate::constants::MAX_RETRIES;
use crate::volt_api::VoltPackage;
use anyhow::{anyhow, ensure, Context, Result};
use colored::Colorize;
use futures::stream::FuturesOrdered;
use futures::TryStreamExt;
use isahc::http::StatusCode;
use isahc::AsyncReadResponseExt;
use isahc::Request;
use isahc::RequestExt;
use semver_rs::Version;
use serde_json::Value;
use ssri::{Algorithm, Integrity};

// Get version from NPM
pub async fn get_version(
    package_name: String,
) -> Result<(String, String, String, Option<VoltPackage>)> {
    let mut retries = 0;

    let count = package_name.matches("@").count();

    if (count == 1 && package_name.contains("/")) || (count == 0 && !package_name.contains("/")) {
        loop {
            let client: Request<&str> =
                Request::get(format!("http://registry.npmjs.org/{}", package_name))
                    .header(
                        "Accept",
                        "application/vnd.npm.install-v1+json; q=1.0, application/json; q=0.8, */*",
                    )
                    .body("")?;

            let mut response = client.send_async().await.with_context(|| {
                format!("failed to fetch {}", package_name.bright_cyan().bold())
            })?;

            match response.status_mut() {
                &mut StatusCode::OK => {
                    let text = response.text().await.context(format!(
                        "failed to deserialize response for {}",
                        package_name.bright_cyan().bold()
                    ))?;

                    match serde_json::from_str::<Value>(&text).unwrap()["dist-tags"]["latest"]
                        .as_str()
                    {
                        Some(latest) => {
                            let num_deps;

                            match serde_json::from_str::<Value>(&text).unwrap()["versions"]
                                ["dependencies"][latest.to_string()]
                            .as_array()
                            {
                                Some(value) => {
                                    num_deps = value.len();
                                }
                                None => {
                                    num_deps = 0;
                                }
                            }

                            let mut package: Option<VoltPackage> = None;

                            match serde_json::from_str::<Value>(&text).unwrap()["versions"][latest]
                                ["dist"]
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
                                            base64::encode(value["sha1"].to_string())
                                        );
                                    }

                                    let integrity: Integrity = hash_string.parse().unwrap();

                                    let algo = integrity.pick_algorithm();

                                    let mut hash = integrity
                                        .hashes
                                        .into_iter()
                                        .find(|h| h.algorithm == algo)
                                        .map(|h| Integrity { hashes: vec![h] })
                                        .map(|i| i.to_hex().1)
                                        .unwrap();

                                    match algo {
                                        Algorithm::Sha1 => {
                                            hash = format!("sha1-{}", hash);
                                        }
                                        Algorithm::Sha512 => {
                                            hash = format!("sha512-{}", hash);
                                        }
                                        _ => {}
                                    }

                                    if num_deps == 0 {
                                        package = Some(VoltPackage {
                                            name: package_name.clone(),
                                            version: latest.to_string(),
                                            tarball: value["tarball"].to_string().replace("\"", ""),
                                            bin: None,
                                            integrity: hash.clone(),
                                            peer_dependencies: None,
                                            dependencies: None,
                                        })
                                    }

                                    return Ok((package_name, latest.to_string(), hash, package));
                                }
                                None => {
                                    return Err(anyhow!(
                                        "Failed to find a hash that matches the specified requirement: {}",
                                        package_name.bright_cyan().bold()
                                    ));
                                }
                            }
                        }
                        None => {
                            return Err(anyhow!(
                                "Failed to request latest version for {}.",
                                package_name.bright_cyan()
                            ));
                        }
                    }
                }
                &mut StatusCode::NOT_FOUND => {
                    if retries == MAX_RETRIES {
                        return Err(anyhow!(
                                "GET {} - {}\n\n{} was not found on the npm registry, or you don't have the permission to request it.",
                                format!("http://registry.npmjs.org/{}", package_name),
                                format!("Not Found ({})", "404".bright_yellow().bold()),
                                package_name,
                            ));
                    }
                }
                _ => {
                    if retries == MAX_RETRIES {
                        return Err(anyhow!(
                                "GET {}: Not Found - {}\n\n{} was not found on the npm registry, or you don't have the permission to request it.",
                                format!("http://registry.npmjs.org/{}", package_name).underline(),
                                response.status().as_str(),
                                package_name
                            ));
                    }
                }
            }

            retries += 1;
        }
    } else {
        if count == 2 && package_name.contains("/") {
            let input_version = package_name.split("@").collect::<Vec<&str>>()[2].to_string();

            let version_requirement = semver_rs::Range::new(&input_version).parse().unwrap();

            loop {
                let name = format!("@{}", input_version);

                let client: Request<&str> = Request::get(format!(
                    "http://registry.npmjs.org/{}",
                    package_name.replace(&name, "")
                ))
                .header(
                    "Accept",
                    "application/vnd.npm.install-v1+json; q=1.0, application/json; q=0.8, */*",
                )
                .body("")?;

                let mut response = client.send_async().await.with_context(|| {
                    format!("failed to fetch {}", package_name.bright_cyan().bold())
                })?;

                match response.status_mut() {
                    &mut StatusCode::OK => {
                        let text = response.text().await.context(format!(
                            "failed to deserialize response for {}",
                            package_name.bright_cyan().bold()
                        ))?;

                        match serde_json::from_str::<Value>(&text).unwrap()["versions"].as_object()
                        {
                            Some(value) => {
                                let mut available_versions = value
                                    .keys()
                                    .filter_map(|k| Version::new(k).parse().ok())
                                    .filter(|v| version_requirement.test(&v))
                                    .collect::<Vec<_>>();

                                ensure!(
                                    !available_versions.is_empty(),
                                    "Failed to find a version that matches the specified requirement: {}",
                                    name.bright_cyan().bold(),
                                );

                                available_versions
                                    .sort_unstable_by(|a, b| a.partial_cmp(b).unwrap().reverse());

                                if available_versions.is_empty() {
                                    return Err(anyhow!(
                                        "Failed to find a version that matches the specified requirement: {}",
                                        name.bright_cyan().bold()
                                    ));
                                }

                                let num_deps;

                                match serde_json::from_str::<Value>(&text).unwrap()["versions"]
                                    ["dependencies"][available_versions[0].to_string()]
                                .as_array()
                                {
                                    Some(value) => {
                                        num_deps = value.len();
                                    }
                                    None => {
                                        num_deps = 0;
                                    }
                                }

                                let mut package: Option<VoltPackage> = None;

                                match serde_json::from_str::<Value>(&text).unwrap()["versions"]
                                    [available_versions[0].to_string()]["dist"]
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
                                                base64::encode(value["sha1"].to_string())
                                            );
                                        }

                                        let integrity: Integrity = hash_string.parse().unwrap();

                                        let algo = integrity.pick_algorithm();

                                        let mut hash = integrity
                                            .hashes
                                            .into_iter()
                                            .find(|h| h.algorithm == algo)
                                            .map(|h| Integrity { hashes: vec![h] })
                                            .map(|i| i.to_hex().1)
                                            .unwrap();

                                        match algo {
                                            Algorithm::Sha1 => {
                                                hash = format!("sha1-{}", hash);
                                            }
                                            Algorithm::Sha512 => {
                                                hash = format!("sha512-{}", hash);
                                            }
                                            _ => {}
                                        }

                                        if num_deps == 0 {
                                            package = Some(VoltPackage {
                                                name: package_name.replace(&name, ""),
                                                version: input_version,
                                                tarball: value["tarball"]
                                                    .to_string()
                                                    .replace("\"", ""),
                                                bin: None,
                                                integrity: hash.clone(),
                                                peer_dependencies: None,
                                                dependencies: None,
                                            })
                                        }
                                        return Ok((
                                            package_name,
                                            available_versions[0].to_string(),
                                            hash,
                                            package,
                                        ));
                                    }
                                    None => {
                                        return Err(anyhow!(
                                            "Failed to find a hash that matches the specified requirement: {}",
                                            name.bright_cyan().bold()
                                        ));
                                    }
                                }
                            }
                            None => {
                                return Err(anyhow!(
                                    "Failed to request versions for {}.",
                                    package_name.bright_cyan()
                                ));
                            }
                        }
                    }
                    &mut StatusCode::NOT_FOUND => {
                        if retries == MAX_RETRIES {
                            return Err(anyhow!(
                                    "GET {} - {}\n\n{} was not found on the npm registry, or you don't have the permission to request it.",
                                    format!("http://registry.npmjs.org/{}", package_name),
                                    format!("Not Found ({})", "404".bright_yellow().bold()),
                                    package_name,
                                ));
                        }
                    }
                    _ => {
                        if retries == MAX_RETRIES {
                            return Err(anyhow!(
                                    "GET {}: Not Found - {}\n\n{} was not found on the npm registry, or you don't have the permission to request it.",
                                    format!("http://registry.npmjs.org/{}", package_name).underline(),
                                    response.status().as_str(),
                                    package_name
                                ));
                        }
                    }
                }

                retries += 1;
            }
        } else if count == 1 && !package_name.contains("/") {
            let input_version = package_name.split("@").collect::<Vec<&str>>()[1].to_string();

            let version_requirement = semver_rs::Range::new(&input_version).parse().unwrap();

            loop {
                let name = format!("@{}", input_version);

                let client: Request<&str> = Request::get(format!(
                    "http://registry.npmjs.org/{}",
                    package_name.replace(&name, "")
                ))
                .header(
                    "Accept",
                    "application/vnd.npm.install-v1+json; q=1.0, application/json; q=0.8, */*",
                )
                .body("")?;

                let mut response = client.send_async().await.with_context(|| {
                    format!("failed to fetch {}", package_name.bright_cyan().bold())
                })?;

                match response.status_mut() {
                    &mut StatusCode::OK => {
                        let text = response.text().await.context(format!(
                            "failed to deserialize response for {}",
                            package_name.bright_cyan().bold()
                        ))?;

                        match serde_json::from_str::<Value>(&text).unwrap()["versions"].as_object()
                        {
                            Some(value) => {
                                let mut available_versions = value
                                    .keys()
                                    .filter_map(|k| Version::new(k).parse().ok())
                                    .filter(|v| version_requirement.test(&v))
                                    .collect::<Vec<_>>();

                                ensure!(
                                    !available_versions.is_empty(),
                                    "Failed to find a version that matches the specified requirement: {}",
                                    name.bright_cyan().bold(),
                                );

                                available_versions
                                    .sort_unstable_by(|a, b| a.partial_cmp(b).unwrap().reverse());

                                if available_versions.is_empty() {
                                    return Err(anyhow!(
                                        "Failed to find a version that matches the specified requirement: {}",
                                        name.bright_cyan().bold()
                                    ));
                                }

                                let num_deps;

                                match serde_json::from_str::<Value>(&text).unwrap()["versions"]
                                    ["dependencies"][available_versions[0].to_string()]
                                .as_array()
                                {
                                    Some(value) => {
                                        num_deps = value.len();
                                    }
                                    None => {
                                        num_deps = 0;
                                    }
                                }

                                let mut package: Option<VoltPackage> = None;

                                match serde_json::from_str::<Value>(&text).unwrap()["versions"]
                                    [available_versions[0].to_string()]["dist"]
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
                                                base64::encode(value["sha1"].to_string())
                                            );
                                        }

                                        let integrity: Integrity = hash_string.parse().unwrap();

                                        let algo = integrity.pick_algorithm();

                                        let mut hash = integrity
                                            .hashes
                                            .into_iter()
                                            .find(|h| h.algorithm == algo)
                                            .map(|h| Integrity { hashes: vec![h] })
                                            .map(|i| i.to_hex().1)
                                            .unwrap();

                                        match algo {
                                            Algorithm::Sha1 => {
                                                hash = format!("sha1-{}", hash);
                                            }
                                            Algorithm::Sha512 => {
                                                hash = format!("sha512-{}", hash);
                                            }
                                            _ => {}
                                        }

                                        if num_deps == 0 {
                                            package = Some(VoltPackage {
                                                name: package_name.replace(&name, ""),
                                                version: input_version,
                                                tarball: value["tarball"]
                                                    .to_string()
                                                    .replace("\"", ""),
                                                bin: None,
                                                integrity: hash.clone(),
                                                peer_dependencies: None,
                                                dependencies: None,
                                            })
                                        }

                                        return Ok((
                                            package_name,
                                            available_versions[0].to_string(),
                                            hash,
                                            package,
                                        ));
                                    }
                                    None => {
                                        return Err(anyhow!(
                                            "Failed to find a hash that matches the specified requirement: {}",
                                            name.bright_cyan().bold()
                                        ));
                                    }
                                }
                            }
                            None => {
                                return Err(anyhow!(
                                    "Failed to request versions for {}.",
                                    package_name.bright_cyan()
                                ));
                            }
                        }
                    }
                    &mut StatusCode::NOT_FOUND => {
                        if retries == MAX_RETRIES {
                            return Err(anyhow!(
                                    "GET {} - {}\n\n{} npm registry, or you don't have the permission to request it.",
                                    format!("http://registry.npmjs.org/{}", package_name),
                                    format!("Not Found ({})", "404".bright_yellow().bold()),
                                    package_name,
                                ));
                        }
                    }
                    _ => {
                        if retries == MAX_RETRIES {
                            return Err(anyhow!(
                                    "GET {}: Not Found - {}\n\n{} was not found on the npm registry, or you don't have the permission to request it.",
                                    format!("http://registry.npmjs.org/{}", package_name).underline(),
                                    response.status().as_str(),
                                    package_name
                                ));
                        }
                    }
                }

                retries += 1;
            }
        } else {
            return Err(anyhow!("an unexpected error happened"));
        }
    }
}

pub async fn get_versions(
    packages: &Vec<String>,
) -> Result<Vec<(String, String, String, Option<VoltPackage>)>> {
    packages
        .to_owned()
        .into_iter()
        .map(get_version)
        .collect::<FuturesOrdered<_>>()
        .try_collect::<Vec<(String, String, String, Option<VoltPackage>)>>()
        .await
}
