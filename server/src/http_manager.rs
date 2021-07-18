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

// Crate Level Imports
use crate::package::Package;

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
pub async fn get_package(name: &str) -> Package {
    let resp = chttp::get_async(format!("http://registry.yarnpkg.com/{}", name))
        .await
        .unwrap();

    let mut body = resp.into_body();
    let body_string = body.text_async().await.unwrap();

    serde_json::from_str(&body_string).unwrap_or_else(|err| {
        println!("{}", err);
        std::process::exit(1);
    })
}
