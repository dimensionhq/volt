use colored::Colorize;
use tokio::fs;
use volt_core::{app::App, command::Command};

pub async fn create_react_app(app_name: String) {
    println!("creating react app: {}", app_name.bright_green());
    fs::create_dir(app_name.clone())
        .await
        .unwrap_or_else(|e| println!("{} {}", "error".bright_red(), e));
    fs::create_dir(format!("{}/src", app_name))
        .await
        .unwrap_or_else(|e| println!("{} {}", "error".bright_red(), e));
    fs::create_dir(format!("{}/public", app_name))
        .await
        .unwrap_or_else(|e| println!("{} {}", "error".bright_red(), e));
    println!("{}", "$ volt init -y".truecolor(147, 148, 148));
    let dir = std::env::current_dir().unwrap().join(&app_name);
    let _ = std::env::set_current_dir(&dir); // Set current directory to $dir as init depends on it
    let mut app = App::initialize();
    app.flags = vec![String::from("-y")];
    volt_init::command::Init::exec(std::sync::Arc::new(app))
        .await
        .unwrap();
    std::env::set_current_dir(std::env::current_dir().unwrap()).unwrap(); // reset current dir
    fs::File::create(format!("{}/README.md", dir.to_str().unwrap()))
        .await
        .unwrap();
}
