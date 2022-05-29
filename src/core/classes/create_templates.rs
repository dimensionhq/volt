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
pub enum Template {
    ReactApp,
    ReactAppTS,
    NextApp,
    NextAppTS,
}

impl Template {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::ReactApp => "react-app",
            Self::ReactAppTS => "react-app-ts",
            Self::NextApp => "next-app",
            Self::NextAppTS => "next-app-ts",
        }
    }
}

impl fmt::Display for Template {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for Template {
    fn default() -> Self {
        Self::ReactApp
    }
}

impl Template {
    pub const _OPTIONS: [&'static str; 4] = [
        Self::ReactApp.as_str(),
        Self::ReactAppTS.as_str(),
        Self::NextApp.as_str(),
        Self::NextAppTS.as_str(),
    ];

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
    pub fn _into_string(self) -> String {
        to_string_pretty(&self).expect("Valid serialization state")
    }
}
