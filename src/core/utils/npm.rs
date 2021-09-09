use miette::DiagnosticResult;

use crate::commands::add::Package;

// use crate::core::utils::constants::MAX_RETRIES;
// use crate::core::utils::errors::VoltError;
// use crate::core::utils::voltapi::VoltPackage;
// use futures::stream::FuturesOrdered;
// use futures::TryStreamExt;
// use isahc::http::StatusCode;
// use isahc::AsyncReadResponseExt;
// use isahc::Request;
// use isahc::RequestExt;
// use miette::DiagnosticResult;
// use semver_rs::Version;
// use serde_json::Value;
// use ssri::{Algorithm, Integrity};

pub async fn parse_versions(packages: &Vec<String>) -> DiagnosticResult<Vec<Package>> {
    let mut parsed: Vec<Package> = vec![];

    for package in packages.iter() {
        let split = package.split("@").map(|s| s.trim()).collect::<Vec<&str>>();
        let length = split.len();

        if length == 1 {
            parsed.push(Package {
                name: split[0].to_string(),
                version: None,
            });
        } else if length == 2 && !package.contains("/") {
            parsed.push(Package {
                name: split[0].to_string(),
                version: Some(split[1].to_string()),
            });
        } else if length == 2 && package.contains("/") {
            parsed.push(Package {
                name: format!("@{}", split[1]),
                version: None,
            });
        } else if length == 3 && package.contains("/") {
            parsed.push(Package {
                name: format!("@{}", split[1]),
                version: Some(split[2].to_string()),
            });
        }
    }

    Ok(parsed)
}
