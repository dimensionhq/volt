use async_trait::async_trait;
use clap::Parser;
use miette::Result;

use crate::cli::{VoltCommand, VoltConfig};

/// List available NodeJS versions
#[derive(Debug, Parser)]
pub struct NodeList {}

#[async_trait]
impl VoltCommand for NodeList {
    // On windows, versions install to C:\Users\[name]\AppData\Roaming\volt\node\[version]
    async fn exec(self, _config: VoltConfig) -> Result<()> {
        let node_path = {
            let datadir = dirs::data_dir().unwrap().join("volt").join("node");
            if !datadir.exists() {
                eprintln!("No NodeJS versions installed!");
                std::process::exit(1);
            };
            datadir
        };

        let mut versions = std::fs::read_dir(&node_path)
            .unwrap()
            .map(|d| d.unwrap().file_name().to_str().unwrap().to_owned())
            .filter(|f| f != "current")
            .collect::<Vec<String>>();

        if versions.is_empty() {
            eprintln!("No NodeJS versions installed!");
            std::process::exit(0);
        }

        // Sort in descending order
        versions.sort_by(|a, b| b.cmp(a));

        for version in versions {
            println!("{version}");
        }

        Ok(())
    }
}
