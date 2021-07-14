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
use utils::app::App;

#[derive(Debug)]
pub enum AppCommand {
    Add,
    Audit,
    Cache,
    Search,
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
    Watch,
    Run,
    Script,
    Update,
    Info,
    Stat,
}

impl FromStr for AppCommand {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "add" => Ok(Self::Add),
            "audit" => Ok(Self::Audit),
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
            "watch" => Ok(Self::Watch),
            "update" => Ok(Self::Update),
            "search" => Ok(Self::Search),
            "info" => Ok(Self::Info),
            "stat" => Ok(Self::Stat),
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
            Self::Add => add::command::Add::help(),
            Self::Audit => audit::command::Audit::help(),
            Self::Cache => cache::command::Cache::help(),
            Self::Compress => compress::command::Compress::help(),
            Self::Clone => clone::command::Clone::help(),
            Self::Create => create::command::Create::help(),
            Self::Deploy => deploy::command::Deploy::help(),
            Self::Help => help::command::Help::help(),
            Self::Init => init::command::Init::help(),
            Self::Install => install::command::Install::help(),
            Self::List => list::command::List::help(),
            Self::Migrate => migrate::command::Migrate::help(),
            Self::Remove => remove::command::Remove::help(),
            Self::Run => run::command::Run::help(),
            Self::Script => scripts::command::Script::help(),
            Self::Fix => fix::command::Fix::help(),
            Self::Watch => watch::command::Watch::help(),
            Self::Update => update::command::Update::help(),
            Self::Search => search::command::Search::help(),
            Self::Info => info::command::Info::help(),
            Self::Stat => stat::command::Stat::help(),
        }
    }

    pub async fn run(&self, app: App) -> Result<()> {
        let app = Arc::new(app);
        match self {
            Self::Add => add::command::Add::exec(app).await,
            Self::Audit => audit::command::Audit::exec(app).await,
            Self::Cache => cache::command::Cache::exec(app).await,
            Self::Clone => clone::command::Clone::exec(app).await,
            Self::Compress => compress::command::Compress::exec(app).await,
            Self::Create => create::command::Create::exec(app).await,
            Self::Deploy => deploy::command::Deploy::exec(app).await,
            Self::Help => help::command::Help::exec(app).await,
            Self::Init => init::command::Init::exec(app).await,
            Self::Install => install::command::Install::exec(app).await,
            Self::List => list::command::List::exec(app).await,
            Self::Migrate => migrate::command::Migrate::exec(app).await,
            Self::Remove => remove::command::Remove::exec(app).await,
            Self::Run => run::command::Run::exec(app).await,
            Self::Script => scripts::command::Script::exec(app).await,
            Self::Fix => fix::command::Fix::exec(app).await,
            Self::Watch => watch::command::Watch::exec(app).await,
            Self::Update => update::command::Update::exec(app).await,
            Self::Search => search::command::Search::exec(app).await,
            Self::Info => info::command::Info::exec(app).await,
            Self::Stat => stat::command::Stat::exec(app).await,
        }
    }
}
