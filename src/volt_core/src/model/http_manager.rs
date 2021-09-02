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

use crate::utils::package::Package;
use isahc::http::StatusCode;
use isahc::AsyncReadResponseExt;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GetPackageError {
    #[error("network request failed with registry")]
    Request(isahc::Error),
    #[error("unable to read network response")]
    IO(io::Error),
    #[error("unable to deserialize network response: {0:?}")]
    Json(serde_json::Error),
}

#[allow(dead_code)]
/// Request a package from `registry.yarnpkg.com`
///
/// Uses `chttp` async implementation to send a `get` request for the package
/// ## Arguments
/// * `name` - Name of the package to request from `registry.yarnpkg.com`
/// ## Examples
/// ```
/// // Await an async response
/// get_package("react").await;
/// ```
/// ## Returns
/// * `Result<Option<Package>, GetPackageError>`
pub async fn get_package(name: &str) -> Result<Option<Package>, GetPackageError> {
    let mut resp = isahc::get_async(format!("http://registry.yarnpkg.com/{}", name))
        .await
        .map_err(GetPackageError::Request)?;

    if !resp.status().is_success() {
        match resp.status() {
            StatusCode::NOT_FOUND => {}
            StatusCode::INTERNAL_SERVER_ERROR => {}
            StatusCode::METHOD_NOT_ALLOWED => {}
            _ => {}
        }
    }

    let body_string = resp.text().await.map_err(GetPackageError::IO)?;
    let package: Package = serde_json::from_str(&body_string).map_err(GetPackageError::Json)?;

    Ok(Some(package))
}
