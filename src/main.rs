/*
 *
 *    Copyright 2021 Volt Contributors
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */
#![allow(unused)]

mod cli;
mod commands;
mod core;

use std::str::FromStr;

use tracing::Level;
use tracing_subscriber::EnvFilter;

use crate::cli::{VoltCli, VoltCommand};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::from_str("volt=info").unwrap()),
        )
        .without_time()
        .init();

    let app = VoltCli::new();

    let code = match app.cmd.exec(app.config).await {
        Err(err) => {
            eprintln!("{}", err.to_string());
            1
        }
        _ => 0,
    };

    std::process::exit(code);
}
