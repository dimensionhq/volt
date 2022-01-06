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

use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;

use std::fmt;

#[derive(Serialize, Deserialize)]
pub enum PackageManager {
    Volt,
    Yarn,
    Pnpm,
    Npm,
}

impl PackageManager {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Volt => "Volt",
            Self::Yarn => "Yarn",
            Self::Pnpm => "pnpm",
            Self::Npm => "npm",
        }
    }
}

impl fmt::Display for PackageManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for PackageManager {
    fn default() -> Self {
        Self::Volt
    }
}

impl PackageManager {
    pub const OPTIONS: [&'static str; 4] = [
        Self::Volt.as_str(),
        Self::Yarn.as_str(),
        Self::Pnpm.as_str(),
        Self::Npm.as_str(),
    ];

    #[allow(dead_code)]
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Volt),
            1 => Some(Self::Yarn),
            2 => Some(Self::Pnpm),
            3 => Some(Self::Npm),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ChangePackageManger {
    pub template: PackageManager,
}

impl ChangePackageManger {
    #[allow(dead_code)]
    pub fn into_string(self) -> String {
        to_string_pretty(&self).unwrap()
    }
}
