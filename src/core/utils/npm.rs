use miette::Result;

use crate::commands::add::PackageInfo;

use crate::core::utils::constants::MAX_RETRIES;
use crate::core::utils::errors::VoltError;
use crate::core::utils::voltapi::VoltPackage;
use futures::stream::FuturesOrdered;
use futures::TryStreamExt;
use isahc::http::StatusCode;
use isahc::AsyncReadResponseExt;
use isahc::Request;
use isahc::RequestExt;

use semver_rs::Version;
use serde_json::Value;
use ssri::{Algorithm, Integrity};

pub fn parse_versions(packages: &Vec<String>) -> Result<Vec<PackageInfo>> {
    let mut parsed: Vec<PackageInfo> = vec![];

    for package in packages.iter() {
        let split = package.split("@").map(|s| s.trim()).collect::<Vec<&str>>();
        let length = split.len();

        if length == 1 {
            parsed.push(PackageInfo {
                name: split[0].to_string(),
                version: None,
            });
        } else if length == 2 && !package.contains("/") {
            parsed.push(PackageInfo {
                name: split[0].to_string(),
                version: Some(split[1].to_string()),
            });
        } else if length == 2 && package.contains("/") {
            parsed.push(PackageInfo {
                name: format!("@{}", split[1]),
                version: None,
            });
        } else if length == 3 && package.contains("/") {
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
    println!("{}", package_name);
    let count = package_name.matches("@").count();

    if (count == 1 && package_name.contains("/")) || (count == 0 && !package_name.contains("/")) {
        loop {
            let client: Request<&str> =
                Request::get(format!("http://registry.npmjs.org/{}", package_name))
                    .header(
                        "Accept",
                        "application/vnd.npm.install-v1+json; q=1.0, application/json; q=0.8, */*",
                    )
                    .body("")
                    .map_err(VoltError::RequestBuilderError)?;

            let mut response = client.send_async().await.map_err(VoltError::NetworkError)?;

            match response.status_mut() {
                &mut StatusCode::OK => {
                    let text = response.text().await.map_err(VoltError::IoTextRecError)?;

                    match serde_json::from_str::<Value>(&text).unwrap()["dist-tags"]["latest"]
                        .as_str()
                    {
                        Some(latest) => {
                            let num_deps;

                            match serde_json::from_str::<Value>(&text).unwrap()["versions"][latest]
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
                                    })?;
                                }
                            }
                        }
                        None => {
                            return Err(VoltError::VersionLookupError { name: package_name })?;
                        }
                    }
                }
                &mut StatusCode::NOT_FOUND => {
                    if retries == MAX_RETRIES {
                        return Err(VoltError::TooManyRequests {
                            url: format!("http://registry.npmjs.org/{}", package_name),
                            package_name: package_name.to_string(),
                        })?;
                    }
                }
                _ => {
                    if retries == MAX_RETRIES {
                        return Err(VoltError::PackageNotFound {
                            url: format!("http://registry.npmjs.org/{}", package_name),
                            package_name: package_name.to_string(),
                        })?;
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
                .body("")
                .map_err(VoltError::RequestBuilderError)?;

                let mut response = client.send_async().await.map_err(VoltError::NetworkError)?;

                match response.status_mut() {
                    &mut StatusCode::OK => {
                        let text = response.text().await.map_err(VoltError::IoTextRecError)?;

                        match serde_json::from_str::<Value>(&text).unwrap()["versions"].as_object()
                        {
                            Some(value) => {
                                let mut available_versions = value
                                    .keys()
                                    .filter_map(|k| Version::new(k).parse().ok())
                                    .filter(|v| version_requirement.test(&v))
                                    .collect::<Vec<_>>();

                                available_versions
                                    .sort_unstable_by(|a, b| a.partial_cmp(b).unwrap().reverse());

                                if available_versions.is_empty() {
                                    return Err(VoltError::VersionLookupError {
                                        name: package_name,
                                    })?;
                                }

                                let num_deps;

                                match serde_json::from_str::<Value>(&text).unwrap()["versions"]
                                    [available_versions[0].to_string()]["dependencies"]
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
                                        })?;
                                    }
                                }
                            }
                            None => {
                                return Err(VoltError::VersionLookupError { name: package_name })?;
                            }
                        }
                    }
                    &mut StatusCode::NOT_FOUND => {
                        if retries == MAX_RETRIES {
                            return Err(VoltError::TooManyRequests {
                                url: format!("http://registry.npmjs.org/{}", package_name),
                                package_name: package_name.to_string(),
                            })?;
                        }
                    }
                    _ => {
                        return Err(VoltError::PackageNotFound {
                            url: format!("http://registry.npmjs.org/{}", package_name),
                            package_name: package_name.to_string(),
                        })?;
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
                .body("")
                .map_err(VoltError::RequestBuilderError)?;

                let mut response = client.send_async().await.map_err(VoltError::NetworkError)?;

                match response.status_mut() {
                    &mut StatusCode::OK => {
                        let text = response.text().await.map_err(VoltError::IoTextRecError)?;

                        match serde_json::from_str::<Value>(&text).unwrap()["versions"].as_object()
                        {
                            Some(value) => {
                                let mut available_versions = value
                                    .keys()
                                    .filter_map(|k| Version::new(k).parse().ok())
                                    .filter(|v| version_requirement.test(&v))
                                    .collect::<Vec<_>>();

                                available_versions
                                    .sort_unstable_by(|a, b| a.partial_cmp(b).unwrap().reverse());

                                if available_versions.is_empty() {
                                    return Err(VoltError::VersionLookupError {
                                        name: package_name,
                                    })?;
                                }

                                let num_deps;

                                match serde_json::from_str::<Value>(&text).unwrap()["versions"]
                                    [available_versions[0].to_string()]["dependencies"]
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
                                        })?;
                                    }
                                }
                            }
                            None => {}
                        }
                    }
                    &mut StatusCode::NOT_FOUND => {
                        if retries == MAX_RETRIES {
                            return Err(VoltError::VersionLookupError { name: package_name })?;
                        }
                    }
                    _ => {
                        if retries == MAX_RETRIES {
                            if retries == MAX_RETRIES {
                                return Err(VoltError::PackageNotFound {
                                    url: format!("http://registry.npmjs.org/{}", package_name),
                                    package_name: package_name.to_string(),
                                })?;
                            }
                        }
                    }
                }

                retries += 1;
            }
        } else {
            return Err(VoltError::UnknownError)?;
        }
    }
}

pub async fn get_versions(
    packages: &Vec<PackageInfo>,
) -> Result<Vec<(PackageInfo, String, VoltPackage, bool)>> {
    packages
        .to_owned()
        .into_iter()
        .map(get_version)
        .collect::<FuturesOrdered<_>>()
        .try_collect::<Vec<(PackageInfo, String, VoltPackage, bool)>>()
        .await
}
