use minreq::{get, Error};

pub fn get_package(name: &str) -> Result<String, Error> {
    let response = get(format!("http://registry.yarnpkg.com/{}", name)).send()?;
    Ok(response.as_str()?.to_string())
}
