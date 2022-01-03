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

#[derive(Serialize, Deserialize, Debug)]
pub enum License {
    Mit,
    Apache2,
    BSD3,
    BSD2,
    Gpl,
    Lgpl,
    Mpl,
    Cddl,
    Unlicense,
    Other,
}

impl License {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Mit => "MIT License",
            Self::Apache2 => "Apache License 2.0",
            Self::BSD3 => "BSD 3-Clause \"New\" or \"Revised\" License",
            Self::BSD2 => "BSD 2-Clause \"Simplified\" or \"FreeBSD\" License",
            Self::Gpl => "GNU General Public License (GPL)",
            Self::Lgpl => "GNU Library or \"Lesser\" General Public License (LGPL)",
            Self::Mpl => "Mozilla Public License 2.0",
            Self::Cddl => "Common Development and Distribution License",
            Self::Unlicense => "The Unlicense",
            Self::Other => "Other",
        }
    }
}

impl fmt::Display for License {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for License {
    fn default() -> Self {
        Self::Mit
    }
}

impl License {
    pub const OPTIONS: [&'static str; 10] = [
        Self::Mit.as_str(),
        Self::Apache2.as_str(),
        Self::BSD3.as_str(),
        Self::BSD2.as_str(),
        Self::Gpl.as_str(),
        Self::Lgpl.as_str(),
        Self::Mpl.as_str(),
        Self::Cddl.as_str(),
        Self::Unlicense.as_str(),
        Self::Other.as_str(),
    ];

    #[allow(dead_code)]
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Mit),
            1 => Some(Self::Apache2),
            2 => Some(Self::BSD3),
            3 => Some(Self::BSD2),
            4 => Some(Self::Gpl),
            5 => Some(Self::Lgpl),
            6 => Some(Self::Mpl),
            7 => Some(Self::Cddl),
            8 => Some(Self::Unlicense),
            9 => Some(Self::Other),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InitData {
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub main: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    pub license: License,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private: Option<bool>,
}

impl InitData {
    // pub fn load(&self) -> InitData {

    // }

    pub fn into_string(self) -> String {
        to_string_pretty(&self).expect("Valid serialization state")
    }
}
