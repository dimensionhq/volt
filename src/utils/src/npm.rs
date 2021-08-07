use crate::constants::MAX_RETRIES;
use anyhow::{anyhow, Context, Result};
use chttp::prelude::Request;
use chttp::RequestExt;
use chttp::{http::StatusCode, ResponseExt};
use colored::Colorize;
use futures::stream::FuturesOrdered;
use futures::TryStreamExt;
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
            let version_requirement = package_name.split("@").collect::<Vec<&str>>()[2];

            loop {
                let client: Request<&str> = Request::get(format!(
                    "http://registry.npmjs.org/{}",
                    package_name
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
                                let versions = value.keys().cloned().collect::<Vec<String>>();

                                
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

            return Err(anyhow!(
                "{} {}: Not Found - {}\n\n{} was not found on the npm registry, or you don't have the permission to request it.",
                "GET".bright_green(),
                format!("http://registry.npmjs.org/{}", package_name).underline(),
                404,
                package_name
            ));
        } else if count == 1 && !package_name.contains("/") {
            let version_requirement = package_name.split("@").collect::<Vec<&str>>()[1];

            return Err(anyhow!(
                "{} {}: Not Found - {}\n\n{} was not found on the npm registry, or you don't have the permission to request it.",
                "GET".bright_cyan(),
                format!("http://registry.npmjs.org/{}", package_name).underline(),
                404,
                package_name
            ));
        } else {
            return Err(anyhow!("something very bad happened lol"));
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
