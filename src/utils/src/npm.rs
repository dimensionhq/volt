use crate::constants::MAX_RETRIES;
use anyhow::{anyhow, ensure, Context, Result};
use chttp::prelude::Request;
use chttp::RequestExt;
use chttp::{http::StatusCode, ResponseExt};
use colored::Colorize;
use futures::stream::FuturesOrdered;
use futures::TryStreamExt;
use semver_rs::Version;
use serde_json::Value;

// Get version from NPM
pub async fn get_version(package_name: String) -> Result<String> {
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
                    let text = response.text_async().await.context(format!(
                        "failed to deserialize response for {}",
                        package_name.bright_cyan().bold()
                    ))?;

                    match serde_json::from_str::<Value>(&text).unwrap()["dist-tags"]["latest"]
                        .as_str()
                    {
                        Some(value) => {
                            return Ok(value.to_string());
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
                        let text = response.text_async().await.context(format!(
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

                                return Ok(available_versions[0].to_string());
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
                        let text = response.text_async().await.context(format!(
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

                                return Ok(available_versions[0].to_string());
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
        } else {
            return Err(anyhow!("an unexpected error happened"));
        }
    }
}

pub async fn get_versions(packages: &Vec<String>) -> Result<Vec<String>> {
    packages
        .to_owned()
        .into_iter()
        .map(get_version)
        .collect::<FuturesOrdered<_>>()
        .try_collect::<Vec<String>>()
        .await
}
