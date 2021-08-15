use colored::Colorize;
use smol::fs;
use utils::app::{App, AppFlag};
use utils::helper::ResultLogErrorExt;
use volt_core::command::Command;

#[allow(dead_code)]
pub async fn create_react_app(app_name: String) {
    println!("creating react app: {}", app_name.bright_green());

    fs::create_dir(&app_name).await.unwrap_and_handle_error();

    fs::create_dir(format!("{}/src", app_name))
        .await
        .unwrap_and_handle_error();

    fs::create_dir(format!("{}/public", app_name))
        .await
        .unwrap_and_handle_error();

    println!("{}", "$ volt init -y".truecolor(147, 148, 148));

    let dir = std::env::current_dir().unwrap().join(&app_name);
    let _ = std::env::set_current_dir(&dir); // Set current directory to $dir as init depends on it
    let mut app = App::initialize().unwrap();
    app.flags = vec![AppFlag::Yes];

    init::command::Init::exec(std::sync::Arc::new(app))
        .await
        .unwrap();

    std::env::set_current_dir(std::env::current_dir().unwrap()).unwrap(); // reset current dir

    fs::File::create(format!("{}/README.md", dir.to_str().unwrap()))
        .await
        .unwrap();
}
