use reqwest::Error;

use crate::classes::package::Package;

pub async fn get_package(name: &str) -> Result<Package, Error> {
    reqwest::get(format!("http://registry.yarnpkg.com/{}", name))
        .await?
        .json()
        .await
}
