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

pub mod models;

use std::env;
use std::process::Stdio;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use actix_web::{get, HttpResponse, Responder};
use async_trait::async_trait;
use once_cell::sync::Lazy;
use tokio::process::Command;
use tokio::time;
pub use models::*;

/// Global cached speedtest result and the instant it was cached.
///
/// Wrapped in a mutex for thread-safe mutable access.
/// Initially empty until the first speedtest run.
pub static LAST_RESULT: Lazy<Mutex<Option<(SpeedTestResult, Instant)>>> = Lazy::new(|| Mutex::new(None));

/// Retrieves the last cached speedtest result, if available.
///
/// # Examples
///
/// ```
/// # use speedtest_statuspage::{SpeedTestResult, get_last_result, set_last_result_for_test, clear_last_result_for_test};
/// # let dummy_result = SpeedTestResult {
/// #     bytes_received: 100,
/// #     bytes_sent: 200,
/// #     download_bps: 1_000_000.0,
/// #     upload_bps: 500_000.0,
/// #     download_mbps: 1.0,
/// #     upload_mbps: 0.5,
/// #     ping_ms: 20.0,
/// #     client: Default::default(),
/// #     server: Default::default(),
/// #     share: None,
/// #     timestamp: "2025-08-07T12:34:56Z".to_string(),
/// # };
/// set_last_result_for_test(dummy_result.clone());
///
/// let result = get_last_result();
/// assert!(result.is_some());
/// assert_eq!(result.unwrap().download_mbps, 1.0);
///
/// clear_last_result_for_test();
/// assert!(get_last_result().is_none());
/// ```

pub fn get_last_result() -> Option<SpeedTestResult> {
    let cache = LAST_RESULT.lock().unwrap();
    cache.as_ref().map(|(result, _)| result.clone())
}

/// Sets the cached speedtest result. Used for testing purposes.
///
/// # Examples
///
/// ```
/// # use speedtest_statuspage::{SpeedTestResult, set_last_result_for_test, get_last_result};
/// # let dummy_result = SpeedTestResult {
/// #     bytes_received: 100,
/// #     bytes_sent: 200,
/// #     download_bps: 1_000_000.0,
/// #     upload_bps: 500_000.0,
/// #     download_mbps: 1.0,
/// #     upload_mbps: 0.5,
/// #     ping_ms: 20.0,
/// #     client: Default::default(),
/// #     server: Default::default(),
/// #     share: None,
/// #     timestamp: "2025-08-07T12:34:56Z".to_string(),
/// # };
/// set_last_result_for_test(dummy_result.clone());
/// let cached = get_last_result().unwrap();
/// assert_eq!(cached.bytes_received, 100);
/// ```

pub fn set_last_result_for_test(result: SpeedTestResult) {
    let mut cache = LAST_RESULT.lock().unwrap();
    *cache = Some((result, Instant::now()));
}

/// Clears the cached speedtest result.
///
/// # Examples
///
/// ```
/// # use speedtest_statuspage::{set_last_result_for_test, clear_last_result_for_test, get_last_result, SpeedTestResult};
/// # let dummy_result = SpeedTestResult::default();
/// set_last_result_for_test(dummy_result);
/// clear_last_result_for_test();
/// assert!(get_last_result().is_none());
/// ```

pub fn clear_last_result_for_test() {
    let mut cache = LAST_RESULT.lock().unwrap();
    *cache = None;
}

/// HTTP GET endpoint `/speed` returns the last cached speedtest result as JSON.
///
/// Returns HTTP 503 Service Unavailable if no result is cached yet.
#[get("/speed")]
pub async fn speedtest() -> impl Responder {
    let cache = LAST_RESULT.lock().unwrap();
    if let Some((cached_result, _timestamp)) = &*cache {
        HttpResponse::Ok().json(cached_result)
    } else {
        HttpResponse::ServiceUnavailable().body("Speedtest result not available yet.")
    }
}

/// Reads the environment variable `INTERVAL_MINUTES` or returns a default of 10 minutes.
///
/// The duration represents how frequently speedtests are run.
pub fn min_frequency_duration() -> Duration {
    let minutes = env::var("INTERVAL_MINUTES")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(10); // default: 10 minutes
    Duration::from_secs(minutes * 60)
}

/// Trait to abstract running the speedtest command.
///
/// Allows mocking speedtest execution for testing.
#[async_trait]
pub trait SpeedtestRunner: Send + Sync {
    /// Runs speedtest and returns the raw JSON string output on success.
    async fn run_speedtest(&self) -> Result<String, String>;
}

/// Real speedtest runner implementation using the `speedtest-cli` binary.
pub struct RealSpeedtestRunner;

#[async_trait]
impl SpeedtestRunner for RealSpeedtestRunner {
    async fn run_speedtest(&self) -> Result<String, String> {
        let output = Command::new("speedtest-cli")
            .arg("--json")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| format!("Failed to run speedtest-cli: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("speedtest-cli failed: {}", stderr))
        }
    }
}

/// Runs the speedtest using the provided runner, parses the JSON output, and caches the result.
///
/// Logs errors to stderr if the command or parsing fails.
pub async fn run_speedtest_and_cache_with_runner(runner: &dyn SpeedtestRunner) {
    match runner.run_speedtest().await {
        Ok(stdout) => match serde_json::from_str::<SpeedTestResponse>(&stdout) {
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

                let mut cache = LAST_RESULT.lock().unwrap();
                *cache = Some((result.clone(), Instant::now()));
                println!("Speedtest updated at {}", result.timestamp);
            }
            Err(e) => eprintln!("Failed to parse speedtest-cli JSON: {}", e),
        },
        Err(e) => eprintln!("{}", e),
    }
}

/// Background async task which schedules periodic speedtest runs.
///
/// The interval between runs is configured by the `INTERVAL_MINUTES` env variable.
pub async fn spawn_speedtest_scheduler() {
    let interval = min_frequency_duration();
    let runner = RealSpeedtestRunner;

    // Run one immediately on startup
    run_speedtest_and_cache_with_runner(&runner).await;

    let mut ticker = time::interval(interval);
    loop {
        ticker.tick().await;
        run_speedtest_and_cache_with_runner(&runner).await;
    }
}

/// Async function to get the cached speedtest result or return an error if not available.
pub async fn get_cached_speedtest_result() -> Result<SpeedTestResult, String> {
    let cache = LAST_RESULT.lock().unwrap();
    if let Some((cached_result, _)) = &*cache {
        Ok(cached_result.clone())
    } else {
        Err("Speedtest result not available yet.".to_string())
    }
}
