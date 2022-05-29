/*
 *    Copyright 2021 Volt Contributors
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

//! Manage local node versions

mod node_install;
mod node_list;
mod node_remove;
mod node_use;

pub use node_install::*;
pub use node_list::*;
pub use node_remove::*;
pub use node_use::*;

use async_trait::async_trait;
use clap::{Parser, Subcommand};
use miette::Result;
use node_semver::Version;
use serde::{Deserialize, Deserializer};

use crate::cli::{VoltCommand, VoltConfig};

#[derive(Deserialize)]
#[serde(untagged)]
enum Lts {
    No(bool),
    Yes(String),
}

impl From<Lts> for Option<String> {
    fn from(val: Lts) -> Self {
        match val {
            Lts::No(_) => None,
            Lts::Yes(x) => Some(x),
        }
    }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Lts::deserialize(deserializer)?.into())
}

#[derive(Deserialize, Debug)]
pub struct NodeVersion {
    pub version: Version,
    #[serde(deserialize_with = "deserialize")]
    pub lts: Option<String>,
    pub files: Vec<String>,
}

/// Manage node versions
#[derive(Debug, Parser)]
pub struct Node {
    #[clap(subcommand)]
    cmd: NodeCommand,
}

#[async_trait]
impl VoltCommand for Node {
    async fn exec(self, config: VoltConfig) -> Result<()> {
        match self.cmd {
            NodeCommand::Use(x) => x.exec(config).await,
            NodeCommand::Install(x) => x.exec(config).await,
            NodeCommand::Remove(x) => x.exec(config).await,
            NodeCommand::List(x) => x.exec(config).await,
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum NodeCommand {
    Use(NodeUse),
    Install(NodeInstall),
    Remove(NodeRemove),
    List(NodeList),
}
