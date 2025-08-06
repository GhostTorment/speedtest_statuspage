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

use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use dotenvy;
use std::env;
use std::process::Stdio;
use tokio::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use once_cell::sync::Lazy;

use models::{SpeedTestResponse, SpeedTestResult};

static LAST_RESULT: Lazy<Mutex<Option<(SpeedTestResult, Instant)>>> = Lazy::new(|| Mutex::new(None));

#[get("/speed")]
async fn speedtest() -> impl Responder {
    let now = Instant::now();
    let min_age = min_frequency_duration();

    // Lock and check cache
    {
        let cache = LAST_RESULT.lock().unwrap();
        if let Some((cached_result, timestamp)) = &*cache {
            if now.duration_since(*timestamp) < min_age {
                return HttpResponse::Ok().json(cached_result);
            }
        }
    }

    // Run `speedtest-cli --json`
    let output = Command::new("speedtest-cli")
        .arg("--json")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await;

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);

            match serde_json::from_str::<SpeedTestResponse>(&stdout) {
                Ok(data) => {
                    let result = SpeedTestResult {
                        bytes_received: data.bytes_received,
                        bytes_sent: data.bytes_sent,
                        download_bps: data.download,
                        upload_bps: data.upload,
                        download_mbps: data.download / 1_000_000.0,
                        upload_mbps: data.upload / 1_000_000.0,
                        ping_ms: data.ping,
                        client: data.client,
                        server: data.server,
                        share: data.share,
                        timestamp: data.timestamp,
                    };

                    // Cache the result
                    let mut cache = LAST_RESULT.lock().unwrap();
                    *cache = Some((result.clone(), now));

                    HttpResponse::Ok().json(result)
                }
                Err(e) => HttpResponse::InternalServerError()
                    .body(format!("Failed to parse speedtest-cli JSON: {}", e)),
            }
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            HttpResponse::InternalServerError()
                .body(format!("speedtest-cli failed: {}", stderr))
        }
        Err(e) => HttpResponse::InternalServerError()
            .body(format!("Failed to run speedtest-cli: {}", e)),
    }
}

fn min_frequency_duration() -> Duration {
    let minutes = env::var("MIN_FREQUENCY_MINUTES")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(10); // default: 10 minutes
    Duration::from_secs(minutes * 60)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let bind_address = env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string());
    let bind_port_str = env::var("BIND_PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_port: u16 = bind_port_str.parse().expect("BIND_PORT must be a valid u16");

    println!("Starting server at http://{}:{}/speed", bind_address, bind_port);

    HttpServer::new(|| App::new().service(speedtest))
        .bind((bind_address.as_str(), bind_port))?
        .run()
        .await
}
