use crate::commands::{add, clean, clone, discord, info, init, login, node, run, search};
use async_trait::async_trait;
use clap::{
    crate_authors, crate_description, crate_name, crate_version, AppSettings, Parser, Subcommand,
};

use super::VoltConfig;

/// A trait to be implemented by subcommands
#[async_trait]
pub trait VoltCommand {
    async fn exec(self, config: VoltConfig) -> miette::Result<()>;
}

/// Volt CLI subcommands
#[derive(Debug, Subcommand)]
pub enum VoltSubCmd {
    Add(add::Add),
    Clone(clone::Clone),
    Init(init::Init),
    Clean(clean::Clean),
    Discord(discord::Discord),
    Search(search::Search),
    Login(login::Login),
    Run(run::Run),
    Info(info::Info),
    Node(node::Node),
}

#[async_trait]
impl VoltCommand for VoltSubCmd {
    async fn exec(self, config: VoltConfig) -> miette::Result<()> {
        match self {
            Self::Add(x) => x.exec(config).await,
            Self::Clone(x) => x.exec(config).await,
            Self::Init(x) => x.exec(config).await,
            Self::Clean(x) => x.exec(config).await,
            Self::Discord(x) => x.exec(config).await,
            Self::Search(x) => x.exec(config).await,
            Self::Login(x) => x.exec(config).await,
            Self::Run(x) => x.exec(config).await,
            Self::Info(x) => x.exec(config).await,
            Self::Node(x) => x.exec(config).await,
        }
    }
}

#[derive(Debug, Parser)]
#[clap(
    name = crate_name!(),
    version = crate_version!(),
    about = crate_description!(),
    author = crate_authors!(),
    global_setting = AppSettings::DisableColoredHelp
)]
#[allow(clippy::module_name_repetitions)]
pub struct VoltCli {
    #[clap(flatten)]
    pub config: VoltConfig,

    #[clap(subcommand)]
    pub cmd: VoltSubCmd,
}

impl VoltCli {
    pub fn new() -> Self {
        Self::parse()
    }
}
