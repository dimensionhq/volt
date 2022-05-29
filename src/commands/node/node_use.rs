use std::path::PathBuf;

use async_trait::async_trait;

use clap::Parser;
use miette::Result;

use crate::cli::{VoltCommand, VoltConfig};

/// Switch current node version
#[derive(Debug, Parser)]
pub struct NodeUse {
    /// Version to use
    version: String,
}

fn get_node_dir() -> PathBuf {
    dirs::data_dir().unwrap().join("volt").join("node")
}

#[async_trait]
impl VoltCommand for NodeUse {
    async fn exec(self, _config: VoltConfig) -> Result<()> {
        #[cfg(target_family = "windows")]
        {
            use_windows(self.version).await;
        }

        #[cfg(target_family = "unix")]
        {
            let node_path = get_node_dir().join(&self.version);

            if node_path.exists() {
                let link_dir = dirs::home_dir().unwrap().join(".local").join("bin");

                let to_install = node_path.join("bin");
                let current = node_path.parent().unwrap().join("current");

                // TODO: Handle file deletion errors
                if current.exists() {
                    // Remove all the currently installed links
                    for f in std::fs::read_dir(&current).unwrap() {
                        let original = f.unwrap().file_name();
                        let installed = link_dir.join(&original);
                        if installed.exists() {
                            std::fs::remove_file(installed).unwrap();
                        }
                    }

                    // Remove the old link
                    std::fs::remove_file(&current).unwrap();

                    // Make a new one to the currently installed version
                    std::os::unix::fs::symlink(&to_install, current).unwrap();
                } else {
                    println!("Installing first version");
                    std::os::unix::fs::symlink(&to_install, current).unwrap();
                }

                // Install all the links for the new version
                for f in std::fs::read_dir(&to_install).unwrap() {
                    let original = f.unwrap().path();
                    let fname = original.file_name().unwrap();
                    let link = link_dir.join(fname);

                    // INFO: DOC: Need to run `rehash` in zsh for the changes to take effect
                    println!("Linking to {:?} from {:?}", link, original);

                    // TODO: Do something with this error
                    let _ = std::fs::remove_file(&link);

                    // maybe ship `vnm` as a shell function to run `volt node use ... && rehash` on
                    // zsh?
                    let _symlink = std::os::unix::fs::symlink(original, link).unwrap();
                }
            } else {
                println!("That version of node is not installed!\nTry \"volt node install {}\" to install that version.", self.version)
            }
        }
        Ok(())
    }
}

pub fn node_use() {
    println!("Hello from node_use.rs!");
}

#[cfg(windows)]
async fn use_windows(version: String) {
    let node_path = get_node_dir().join(&version).join("node.exe");
    let path = Path::new(&node_path);

    if path.exists() {
        println!("Using version {}", version);

        let link_dir = dirs::data_dir()
            .unwrap()
            .join("volt")
            .join("bin")
            .into_os_string()
            .into_string()
            .unwrap();

        let link_file = dirs::data_dir()
            .unwrap()
            .join("volt")
            .join("bin")
            .join("node.exe");
        let link_file = Path::new(&link_file);

        if link_file.exists() {
            fs::remove_file(link_file).await.unwrap();
        }

        let newfile = std::fs::copy(node_path, link_file);

        match newfile {
            Ok(_) => {}
            Err(_) => {
                println!("Sorry, something went wrong.");
                return;
            }
        }

        let vfpath = dirs::data_dir().unwrap().join("volt").join("current");
        let vfpath = Path::new(&vfpath);
        let vfile = std::fs::write(vfpath, version);

        let path = env::var("PATH").unwrap();
        if !path.contains(&link_dir) {
            let command = format!("[Environment]::SetEnvironmentVariable('Path', [Environment]::GetEnvironmentVariable('Path', 'User') + '{}', 'User')", &link_dir);
            Command::new("Powershell")
                .args(&["-Command", &command])
                .output()
                .unwrap();
            println!("PATH environment variable updated.\nYou will need to restart your terminal for changes to apply.");
        }
    } else {
        println!("That version of node is not installed!\nTry \"volt node install {}\" to install that version.", version);
    }
}
