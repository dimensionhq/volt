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
pub enum Template {
    ReactApp,
    ReactAppTS,
    NextApp,
    NextAppTS,
}

impl fmt::Display for Template {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ReactApp => write!(f, "react-app"),
            Self::ReactAppTS => write!(f, "react-app-ts"),
            Self::NextApp => write!(f, "next-app"),
            Self::NextAppTS => write!(f, "next-app-ts"),
        }
    }
}

impl Default for Template {
    fn default() -> Self {
        Self::ReactApp
    }
}

impl Template {
    #[allow(dead_code)]
    pub fn options() -> Vec<String> {
        vec![
            Self::ReactApp.to_string(),
            Self::ReactAppTS.to_string(),
            Self::NextApp.to_string(),
            Self::NextAppTS.to_string(),
        ]
    }

    #[allow(dead_code)]
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::ReactApp),
            1 => Some(Self::ReactAppTS),
            2 => Some(Self::NextApp),
            3 => Some(Self::NextAppTS),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct CreateTemplate {
    pub template: Template,
}

impl CreateTemplate {
    #[allow(dead_code)]
    pub fn dump(&self) -> String {
        to_string_pretty(&self).unwrap()
    }
}
