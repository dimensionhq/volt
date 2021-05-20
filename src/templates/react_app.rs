use std::fs;
use colored::Colorize;

pub fn create_react_app(app_name: String) {
    println!("creating react app: {}", app_name.bright_green());
    fs::create_dir(app_name).unwrap_or_else(|e| println!("{} {}", "error".bright_red(), e));
}