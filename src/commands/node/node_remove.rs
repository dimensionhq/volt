use std::path::PathBuf;

use crate::cli::{VoltCommand, VoltConfig};
use async_trait::async_trait;
use clap::{CommandFactory, ErrorKind, Parser};
use miette::{IntoDiagnostic, Result};

/// Uninstall a specified version of node
#[derive(Debug, Parser)]
pub struct NodeRemove {
    /// Versions to remove
    versions: Vec<String>,
}

fn get_node_dir() -> PathBuf {
    dirs::data_dir().unwrap().join("volt").join("node")
}

// #[cfg(unix)]
#[async_trait]
impl VoltCommand for NodeRemove {
    async fn exec(self, _config: VoltConfig) -> Result<()> {
        if self.versions.is_empty() {
            NodeRemove::command()
                .error(
                    ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand,
                    "Must have at least one version",
                )
                .exit();
        }

        let node_dir = get_node_dir();

        let current_dir = if node_dir.join("current").exists() {
            let curr = std::fs::canonicalize(node_dir.join("current"))
                .unwrap()
                .parent()
                .unwrap()
                .to_owned();
            Some(curr)
        } else {
            None
        };

        let current_version = current_dir
            .as_ref()
            .map(|dir| dir.file_name().unwrap().to_str().unwrap());

        for v in self.versions {
            let version_dir = node_dir.join(&v);

            if !version_dir.exists() {
                eprintln!("Version {v} not installed");
                continue;
            }

            if matches!(current_version, Some(ver) if ver == v) {
                let current_dir = current_dir.as_ref().unwrap();
                let current_bin = std::fs::read_dir(current_dir.join("bin")).unwrap();

                // Remove all the installed symlinks
                for binary in current_bin {
                    let b = binary.unwrap();
                    let result =
                        std::fs::remove_file(dirs::executable_dir().unwrap().join(b.file_name()));

                    match result {
                        Ok(_) => {}
                        Err(e) => return Err(e).into_diagnostic(),
                    }
                }

                let result = std::fs::remove_file(node_dir.join("current"));

                match result {
                    Ok(_) => {}
                    Err(e) => return Err(e).into_diagnostic(),
                }
            }

            // Always remove the version directory, regardless of current version status
            let result = std::fs::remove_dir_all(node_dir.join(v));

            match result {
                Ok(_) => {}
                Err(e) => return Err(e).into_diagnostic(),
            }
        }
        Ok(())
    }
}
