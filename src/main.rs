//! # speedtest-statuspage
//!
//! A utility application to serve speedtest results over an HTTP endpoint.
//!
//! ## Disclaimer
//! This project is not affiliated with, endorsed by, or sponsored by Ookla. (Ookla®).
//! All trademarks and copyrights belong to their respective owners.

// Copyright (c) 2025 Jak Bracegirdle
//
// This file is part of the speedtest_statuspage crate.
//
// Licensed under the Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0>
// or the MIT license <http://opensource.org/licenses/MIT>, at your option.
// This file may not be copied, modified, or distributed except according to those terms.

mod models;

use actix_web::{App, HttpServer};
use dotenvy;
use std::env;
use speedtest_statuspage::{spawn_speedtest_scheduler, speedtest};

/// Main entrypoint starts the Actix-web server and the periodic speedtest runner.
///
/// Binds to `BIND_ADDRESS` and `BIND_PORT` environment variables or defaults.
///
/// # Panics
///
/// Panics if `BIND_PORT` cannot be parsed as a valid `u16`.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let bind_address = env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string());
    let bind_port_str = env::var("BIND_PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_port: u16 = bind_port_str.parse().expect("BIND_PORT must be a valid u16");

    // Spawn the periodic speedtest updater
    tokio::spawn(spawn_speedtest_scheduler());

    println!("Starting server at http://{}:{}/speed", bind_address, bind_port);

    HttpServer::new(|| App::new().service(speedtest))
        .bind((bind_address.as_str(), bind_port))?
        .run()
        .await
}