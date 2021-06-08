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

// Std Imports
use std::fmt;

// Library Imports
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;

#[derive(Serialize, Deserialize)]
pub enum PackageManager {
    Volt,
    Yarn,
    Pnpm,
    Npm,
}

impl fmt::Display for PackageManager {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Volt => write!(f, "Volt"),
            Self::Yarn => write!(f, "Yarn"),
            Self::Pnpm => write!(f, "pnpm"),
            Self::Npm => write!(f, "npm"),
        }
    }
}

impl Default for PackageManager {
    fn default() -> Self {
        Self::Volt
    }
}

impl PackageManager {
    #[allow(dead_code)]
    pub fn options() -> Vec<String> {
        vec![
            Self::Volt.to_string(),
            Self::Yarn.to_string(),
            Self::Pnpm.to_string(),
            Self::Npm.to_string(),
        ]
    }

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
    pub fn dump(&self) -> String {
        to_string_pretty(&self).unwrap()
    }
}
