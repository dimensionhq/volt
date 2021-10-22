/*
    Copyright 2021 Volt Contributors

    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at

        http://www.apache.org/licenses/LICENSE-2.0

    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.
*/

//! Remove a package from your direct dependencies.

use crate::{core::VERSION, App, Command};

use async_trait::async_trait;
use colored::Colorize;
use miette::Result;

use std::sync::Arc;

/// Struct implementation for the `Remove` command.
pub struct Create;

#[async_trait]
impl Command for Create {
    /// Display a help menu for the `volt create` command.
    fn help() -> String {
        format!(
            r#"volt {}

            Creates a project from a template.

            Usage: {} {} {} {}

            Options:

            {} {} Output the version number.
            {} {} Output verbose messages on internal operations."#,
            VERSION.bright_green().bold(),
            "volt".bright_green().bold(),
            "create".bright_purple(),
            "[template]".white(),
            "[flags]".white(),
            "--version".blue(),
            "(-ver)".yellow(),
            "--verbose".blue(),
            "(-v)".yellow()
        )
    }

    /// Execute the `volt create` command
    ///
    /// Removes a package from your direct dependencies.
    /// ## Arguments
    /// * `app` - Instance of the command (`Arc<App>`)
    /// * `template` - Template to create with
    /// * `flags` - List of flags passed in through the CLI (`Vec<String>`)
    /// ## Examples
    /// ```
    /// // Remove a package from your direct dependencies with logging level verbose
    /// // .exec() is an async call so you need to await it
    /// Create.exec(app, vec![], vec!["--verbose"]).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`

    #[allow(unused)]
    async fn exec(app: Arc<App>) -> Result<()> {
        // let args = app.args.clone();
        // let templates: Vec<String> = Template::options();

        // let mut template: String = String::new();

        // let mut app_name: String = String::new();
        // if args.len() == 1 {
        //     let select = Select {
        //         message: String::from("Template"),
        //         paged: true,
        //         selected: Some(1),
        //         items: templates.clone(),
        //     };

        //     let selected = select.run().unwrap_or_else(|err| {
        //         error!("{}", err.to_string());
        //         process::exit(1);
        //     });

        //     template = Template::from_index(selected).unwrap().to_string();
        // } else {
        //     let _template = &args[1];
        //     if templates.contains(_template) {
        //         template = _template.to_string();
        //     } else {
        //         error!("Template {} doesn't exist!", _template.bright_blue());
        //         process::exit(1);
        //     }
        // }

        // if args.len() > 1 {
        //     app_name = Input::new()
        //         .with_prompt("App name")
        //         .with_initial_text("")
        //         .default("my-app".into())
        //         .interact_text()
        //         .unwrap();

        //     if app_name.is_empty() {
        //         error!("Invalid app name!");
        //         process::exit(1);
        //     }
        // } else {
        //     let _app_name = &args[1];
        //     app_name = _app_name.to_string();
        // }

        // let template_name = template.split('-').collect::<Vec<&str>>()[0];
        // let version = "create-".to_owned() + template_name;
        // let package_json = get_package(&version).await.unwrap().unwrap_or_else(|| {
        //     println!(
        //         "{} Could not find template for {}",
        //         "error".red().bold(),
        //         template_name
        //     );
        //     exit(1)
        // });
        // // For dev checking
        // let v = package_json
        //     .versions
        //     .get(package_json.dist_tags.get("latest").unwrap())
        //     .unwrap_or_else(|| {
        //         println!(
        //             "{} Could not find template version for {}",
        //             "error".red().bold(),
        //             template_name
        //         );
        //         exit(1)
        //     });

        // println!("HANDLE THIS");
        // let tarball_file = utils::download_tarball_create(&app, &package_json, &version)
        //     .await
        //     .unwrap();
        // let gz_decoder = GzDecoder::new(File::open(tarball_file).unwrap());

        // let mut archive = Archive::new(gz_decoder);
        // let mut dir = std::env::current_dir().unwrap();

        // archive.unpack(&dir.join(app_name)).unwrap();

        Ok(())
    }
}
