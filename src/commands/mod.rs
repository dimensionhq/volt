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

use anyhow::Result;
use async_trait::async_trait;
use std::{str::FromStr, sync::Arc};

use crate::utils::App;

pub mod add;
pub mod help;
pub mod init;
pub mod install;
pub mod remove;

#[derive(Debug)]
pub enum AppCommand {
    Add,
    Help,
    Init,
    Install,
    Remove,
}

impl FromStr for AppCommand {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "add" => Ok(Self::Add),
            "help" => Ok(Self::Help),
            "init" => Ok(Self::Init),
            "install" => Ok(Self::Install),
            "remove" => Ok(Self::Remove),
            _ => Err(()),
        }
    }
}

impl AppCommand {
    pub fn current() -> Option<Self> {
        match std::env::args().nth(1) {
            Some(cmd) => Self::from_str(cmd.as_str()).ok(),
            None => None,
        }
    }

    pub fn command(&self) -> Box<dyn Command> {
        match self {
            Self::Add => Box::new(add::Add),
            Self::Help => Box::new(help::Help),
            Self::Init => Box::new(init::Init),
            Self::Install => Box::new(install::Install),
            Self::Remove => Box::new(remove::Remove),
        }
    }
}

#[async_trait]
pub trait Command {
    fn help(&self) -> String;

    async fn exec(&self, app: Arc<App>, args: Vec<String>, flags: Vec<String>) -> Result<()>;
}
