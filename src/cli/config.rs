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

use crate::core::utils::{enable_ansi_support, errors::VoltError};

use clap::ArgMatches;
use clap::Parser;
use dirs::home_dir;
use package_spec::{parse_package_spec, PackageSpec};
use sha1::Digest;
use sha2::Sha512;
use ssri::{Algorithm, Integrity};
use std::{env, path::PathBuf};

#[derive(Debug, Clone, Parser)]
pub struct VoltConfig {
    /// Path to current working directory
    #[clap(short, long)]
    cwd: Option<PathBuf>,
}

impl VoltConfig {
    pub const OS: &'static str = env::consts::OS;
    pub const VOLT_HOME: &'static str = ".volt";
    pub const VOLT_LOCK: &'static str = "volt.lock";

    pub fn home(&self) -> miette::Result<PathBuf> {
        Ok(dirs::home_dir().ok_or(VoltError::GetHomeDirError)?)
    }

    /// Return the current directory (defaults to `.` if not provided)
    pub fn cwd(&self) -> miette::Result<PathBuf> {
        Ok(self.cwd.to_owned().unwrap_or({
            env::current_dir().map_err(|e| VoltError::EnvironmentError {
                env: "CURRENT_DIRECTORY".to_string(),
                source: e,
            })?
        }))
    }

    /// Path to the volt lockfile (defaults to `./volt.lock`)
    pub fn lockfile(&self) -> miette::Result<PathBuf> {
        Ok(self.cwd()?.join(Self::VOLT_LOCK))
    }

    /// Path to the `node_modules` directory (defaults to `./node_modules`)
    pub fn node_modules(&self) -> miette::Result<PathBuf> {
        Ok(self.cwd()?.join("node_modules"))
    }

    /// Path to the config directory (defaults to `~/.volt`)
    pub fn volt_home(&self) -> miette::Result<PathBuf> {
        Ok(self.home()?.join(Self::VOLT_HOME))
    }

    /// Calculate the hash of a tarball
    ///
    /// ## Examples
    /// ```rs
    /// calc_hash(bytes::Bytes::new(), ssri::Algorithm::Sha1)?;
    /// ```
    /// ## Returns
    /// * Result<String>
    pub fn calc_hash(data: &bytes::Bytes, algorithm: Algorithm) -> miette::Result<String> {
        let integrity;

        if algorithm == Algorithm::Sha1 {
            let hash = ssri::IntegrityOpts::new()
                .algorithm(Algorithm::Sha1)
                .chain(&data)
                .result();

            integrity = format!("sha1-{}", hash.to_hex().1);
        } else {
            integrity = ssri::IntegrityOpts::new()
                .algorithm(Algorithm::Sha512)
                .chain(&data)
                .result()
                .to_string();
        }

        Ok(integrity)
    }
}
