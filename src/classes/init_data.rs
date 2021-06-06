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

#[derive(Serialize, Deserialize, Debug)]
pub enum License {
    MIT = 0,
    Apache2 = 1,
    BSD3,
    BSD2,
    GPL,
    LGPL,
    MPL,
    CDDL,
    Unlicense,
    Other,
}

impl fmt::Display for License {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::MIT => write!(f, "MIT License"),
            Self::Apache2 => write!(f, "Apache License 2.0"),
            Self::BSD3 => write!(f, "BSD 3-Clause \"New\" or \"Revised\" License"),
            Self::BSD2 => write!(f, "BSD 2-Clause \"Simplified\" or \"FreeBSD\" License"),
            Self::GPL => write!(f, "GNU General Public License (GPL)"),
            Self::LGPL => write!(f, "GNU Library or \"Lesser\" General Public License (LGPL)"),
            Self::MPL => write!(f, "Mozilla Public License 2.0"),
            Self::CDDL => write!(f, "Common Development and Distribution License"),
            Self::Unlicense => write!(f, "The Unlicense"),
            Self::Other => write!(f, "Other"),
        }
    }
}

impl Default for License {
    fn default() -> Self {
        Self::MIT
    }
}

impl License {
    #[allow(dead_code)]
    pub fn options() -> Vec<String> {
        vec![
            Self::MIT.to_string(),
            Self::Apache2.to_string(),
            Self::BSD3.to_string(),
            Self::BSD2.to_string(),
            Self::GPL.to_string(),
            Self::LGPL.to_string(),
            Self::MPL.to_string(),
            Self::CDDL.to_string(),
            Self::Unlicense.to_string(),
            Self::BSD3.to_string(),
        ]
    }

    #[allow(dead_code)]
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::MIT),
            1 => Some(Self::Apache2),
            2 => Some(Self::BSD3),
            3 => Some(Self::BSD2),
            4 => Some(Self::GPL),
            5 => Some(Self::LGPL),
            6 => Some(Self::MPL),
            7 => Some(Self::CDDL),
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

    #[allow(dead_code)]
    pub fn dump(&self) -> String {
        to_string_pretty(&self).unwrap()
    }
}
