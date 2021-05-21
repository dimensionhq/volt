use colored::Colorize;
use std::fs;

use crate::commands::Command;

pub async fn create_react_app(app_name: String) {
    println!("creating react app: {}", app_name.bright_green());
    fs::create_dir(app_name.clone()).unwrap_or_else(|e| println!("{} {}", "error".bright_red(), e));
    fs::create_dir(format!("{}/src", app_name))
        .unwrap_or_else(|e| println!("{} {}", "error".bright_red(), e));
    fs::create_dir(format!("{}/public", app_name))
        .unwrap_or_else(|e| println!("{} {}", "error".bright_red(), e));
    println!("{}", "$ volt init -y".truecolor(147, 148, 148));
    let dir = std::env::current_dir().unwrap().join(&app_name);
    let _ = std::env::set_current_dir(&dir); // Set current directory to $dir as init depends on it
    let mut app = crate::utils::App::initialize();
    app.flags = vec![String::from("-y")];
    crate::commands::init::Init::exec(std::sync::Arc::new(app)).await;
    std::env::set_current_dir(std::env::current_dir().unwrap()); // reset current dir
    fs::File::create(format!("{}/README.md", dir.to_str().unwrap())).unwrap();
}
