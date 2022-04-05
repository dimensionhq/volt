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
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod cli;
mod commands;
mod core;

use std::{io::stdin, str::FromStr, time::Instant};

use tracing::Level;
use tracing_subscriber::EnvFilter;

use crate::cli::{VoltCli, VoltCommand};

//#[tokio::main(worker_threads = 6)]
//#[tokio::main(flavor = "current_thread")]
fn main() -> miette::Result<()> {
    let body = async {
        tracing_subscriber::fmt()
            .with_max_level(Level::TRACE)
            .with_env_filter(
                EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| EnvFilter::from_str("volt=info").unwrap()),
            )
            .without_time()
            .init();

        if cfg!(windows) {
            core::utils::enable_ansi_support().unwrap();
        }

        let start = Instant::now();

        let app = VoltCli::new();

        app.cmd.exec(app.config).await?;

        println!("Finished in {:.2}s", start.elapsed().as_secs_f32());

        Ok(())
    };

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(6)
        .max_blocking_threads(6)
        .thread_name("volt")
        .enable_all()
        .build()
        .expect("Failed to build the runtime")
        .block_on(body)

    /*
     *tokio::runtime::Builder::new_multi_thread()
     *    .worker_threads(2)
     *    .enable_all()
     *    .build()
     *    .expect("Failed building the Runtime")
     *    .block_on(body)
     */
}
