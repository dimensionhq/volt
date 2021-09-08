use std::collections::HashMap;

use miette::DiagnosticResult;

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

pub async fn parse_versions(packages: &Vec<String>) -> DiagnosticResult<HashMap<String, String>> {
    let mut parsed: HashMap<String, String> = HashMap::new();

    for package in packages.iter() {
        let split = package.split("@").map(|s| s.trim()).collect::<Vec<&str>>();
        let length = split.len();

        if length == 1 {
            parsed.insert(split[0].to_string(), String::new());
        } else if length == 2 && !package.contains("/") {
            parsed.insert(split[0].to_string(), split[1].to_string());
        } else if length == 2 && package.contains("/") {
            parsed.insert(format!("@{}", split[1]), String::new());
        } else if length == 3 && package.contains("/") {
            parsed.insert(format!("@{}", split[1]), split[2].to_string());
        }
    }

    Ok(parsed)
}
