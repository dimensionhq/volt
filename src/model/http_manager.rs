use std::io;

use crate::classes::package::Package;

#[derive(Debug)]
pub enum GetPackageError {
    RequestError(chttp::Error),
    IOError(io::Error),
    JSONError(serde_json::Error),
}

pub async fn get_package(name: &str) -> Result<Package, GetPackageError> {
    let mut body = chttp::get_async(format!("http://registry.yarnpkg.com/{}", name))
        .await
        .map_err(GetPackageError::RequestError)?
        .into_body();

    let body_string = body.text().map_err(GetPackageError::IOError)?;

    serde_json::from_str(&body_string).map_err(GetPackageError::JSONError)
}
