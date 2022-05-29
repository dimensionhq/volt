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
    async fn exec(self, config: VoltConfig) -> Result<()> {
        let node_path = {
            let datadir = dirs::data_dir().unwrap().join("volt").join("node");
            if !datadir.exists() {
                eprintln!("No NodeJS versions installed!");
                std::process::exit(1);
            };
            datadir
        };

        let files = std::fs::read_dir(&node_path)
            .unwrap()
            .map(|d| {
                d.unwrap()
                    .path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned()
            })
            .filter(|f| f != "current")
            .collect::<Vec<String>>();

        if files.is_empty() {
            eprintln!("No NodeJS versions installed!");
            std::process::exit(1);
        }

        for file in files {
            println!("{file}");
        }

        Ok(())
    }
}
