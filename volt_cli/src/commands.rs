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

use std::str::FromStr;
use std::sync::Arc;

use anyhow::Result;
use volt_core::command::Command;
use volt_utils::app::App;

#[derive(Debug)]
pub enum AppCommand {
    Add,
    Cache,
    Clone,
    Compress,
    Create,
    Deploy,
    Help,
    Init,
    Install,
    List,
    Migrate,
    Remove,
    Fix,
    Run,
    Unknown,
}

impl FromStr for AppCommand {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "add" => Ok(Self::Add),
            "cache" => Ok(Self::Cache),
            "clone" => Ok(Self::Clone),
            "compress" => Ok(Self::Compress),
            "create" => Ok(Self::Create),
            "deploy" => Ok(Self::Deploy),
            "help" => Ok(Self::Help),
            "init" => Ok(Self::Init),
            "install" => Ok(Self::Install),
            "list" => Ok(Self::List),
            "migrate" => Ok(Self::Migrate),
            "remove" => Ok(Self::Remove),
            "run" => Ok(Self::Run),
            "fix" => Ok(Self::Fix),
            _ => Err(()),
        }
    }
}

impl AppCommand {
    pub fn current() -> Option<Self> {
        if std::env::args().len() == 1 {
            return Some(Self::Help);
        }

        match std::env::args().nth(1) {
            Some(cmd) => Self::from_str(cmd.as_str()).ok(),
            None => None,
        }
    }

    pub fn help(&self) -> String {
        match self {
            Self::Add => volt_add::command::Add::help(),
            Self::Cache => volt_cache::command::Cache::help(),
            Self::Compress => volt_compress::command::Compress::help(),
            Self::Clone => volt_clone::command::Clone::help(),
            Self::Create => volt_create::command::Create::help(),
            Self::Deploy => volt_deploy::command::Deploy::help(),
            Self::Help => volt_help::command::Help::help(),
            Self::Init => volt_init::command::Init::help(),
            Self::Install => volt_install::command::Install::help(),
            Self::List => volt_list::command::List::help(),
            Self::Migrate => volt_migrate::command::Migrate::help(),
            Self::Remove => volt_remove::command::Remove::help(),
            Self::Run => volt_run::command::Run::help(),
            Self::Unknown => volt_unknown::command::Unknown::help(),
            Self::Fix => volt_fix::command::Fix::help(),
        }
    }

    pub async fn run(&self, app: App) -> Result<()> {
        let app = Arc::new(app);
        match self {
            Self::Add => volt_add::command::Add::exec(app).await,
            Self::Cache => volt_cache::command::Cache::exec(app).await,
            Self::Clone => volt_clone::command::Clone::exec(app).await,
            Self::Compress => volt_compress::command::Compress::exec(app).await,
            Self::Create => volt_create::command::Create::exec(app).await,
            Self::Deploy => volt_deploy::command::Deploy::exec(app).await,
            Self::Help => volt_help::command::Help::exec(app).await,
            Self::Init => volt_init::command::Init::exec(app).await,
            Self::Install => volt_install::command::Install::exec(app).await,
            Self::List => volt_list::command::List::exec(app).await,
            Self::Migrate => volt_migrate::command::Migrate::exec(app).await,
            Self::Remove => volt_remove::command::Remove::exec(app).await,
            Self::Run => volt_run::command::Run::exec(app).await,
            Self::Unknown => volt_unknown::command::Unknown::exec(app).await,
            Self::Fix => volt_fix::command::Fix::exec(app).await,
        }
    }
}
