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
use async_trait::async_trait;
use dotenvy;
use once_cell::sync::Lazy;
use std::env;
use std::process::Stdio;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::{process::Command, time};

use models::{SpeedTestResponse, SpeedTestResult};

/// Global cached speedtest result and the instant it was cached.
///
/// Wrapped in a mutex for thread-safe mutable access.
/// Initially empty until the first speedtest run.
static LAST_RESULT: Lazy<Mutex<Option<(SpeedTestResult, Instant)>>> = Lazy::new(|| Mutex::new(None));

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

#[cfg(test)]
pub(crate) fn get_last_result() -> Option<SpeedTestResult> {
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

#[cfg(test)]
pub(crate) fn set_last_result_for_test(result: SpeedTestResult) {
    let mut cache = LAST_RESULT.lock().unwrap();
    *cache = Some((result, std::time::Instant::now()));
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

#[cfg(test)]
pub fn clear_last_result_for_test() {
    let mut cache = LAST_RESULT.lock().unwrap();
    *cache = None;
}

/// HTTP GET endpoint `/speed` returns the last cached speedtest result as JSON.
///
/// Returns HTTP 503 Service Unavailable if no result is cached yet.
#[get("/speed")]
async fn speedtest() -> impl Responder {
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
fn min_frequency_duration() -> Duration {
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
async fn spawn_speedtest_scheduler() {
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

#[cfg(test)]
mod tests {
    //! Integration and unit tests for `speedtest-statuspage` internals.
    //!
    //! These tests verify the behavior of the cached speedtest results,
    //! the HTTP endpoint `/speed`, and cache manipulation helpers.
    //!
    //! Note: Some tests require `actix-web` async runtime and thus
    //! are annotated with `#[actix_web::test]` or `#[tokio::test]`.
    //!
    //! The `serial_test::serial` attribute ensures tests sharing global
    //! state run sequentially to avoid race conditions.

    use super::*;
    use actix_web::{test, http, App};
    use serial_test::serial;

    /// Creates a dummy `SpeedTestResult` with fixed example values
    /// for use in tests.
    fn dummy_result() -> SpeedTestResult {
        SpeedTestResult {
            bytes_received: 100,
            bytes_sent: 200,
            download_bps: 1_000_000.0,
            upload_bps: 500_000.0,
            download_mbps: 1.0,
            upload_mbps: 0.5,
            ping_ms: 20.0,
            client: Default::default(),
            server: Default::default(),
            share: None,
            timestamp: "2025-08-07T12:34:56Z".to_string(),
        }
    }

    /// Test that the `/speed` endpoint returns HTTP 503 Service Unavailable
    /// when there is no cached speedtest result.
    ///
    /// ```rust
    /// # use speedtest_statuspage::{clear_last_result_for_test};
    /// # // This example shows intent but is not a real doctest (needs actix runtime)
    /// clear_last_result_for_test();
    /// // HTTP GET /speed should return 503 Service Unavailable
    /// ```
    #[actix_web::test]
    #[serial]
    async fn speedtest_returns_service_unavailable_when_no_cache() {
        clear_last_result_for_test();

        let app = test::init_service(App::new().service(speedtest)).await;
        let req = test::TestRequest::get().uri("/speed").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), http::StatusCode::SERVICE_UNAVAILABLE);

        let body = test::read_body(resp).await;
        assert_eq!(body, "Speedtest result not available yet.");
    }

    /// Test that the `/speed` endpoint returns the cached speedtest result
    /// as JSON with HTTP 200 OK status.
    ///
    /// ```rust
    /// # use speedtest_statuspage::{set_last_result_for_test, clear_last_result_for_test};
    /// # // This example shows intent but is not a real doctest (needs actix runtime)
    /// set_last_result_for_test(dummy_result());
    /// // HTTP GET /speed should return 200 OK with JSON body containing cached data
    /// clear_last_result_for_test();
    /// ```
    #[actix_web::test]
    #[serial]
    async fn speedtest_returns_cached_result() {
        set_last_result_for_test(dummy_result());

        let app = test::init_service(App::new().service(speedtest)).await;
        let req = test::TestRequest::get().uri("/speed").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), http::StatusCode::OK);

        let body = test::read_body(resp).await;
        let result: SpeedTestResult = serde_json::from_slice(&body).unwrap();

        assert_eq!(result.download_mbps, 1.0);
        assert_eq!(result.bytes_received, 100);

        clear_last_result_for_test();
    }

    /// Tests the cache helper functions `set_last_result_for_test`,
    /// `get_last_result`, and `clear_last_result_for_test` for expected behavior.
    ///
    /// ```rust
    /// # use speedtest_statuspage::{set_last_result_for_test, get_last_result, clear_last_result_for_test};
    /// # let res = dummy_result();
    /// clear_last_result_for_test();
    /// assert!(get_last_result().is_none());
    ///
    /// set_last_result_for_test(res.clone());
    /// assert_eq!(get_last_result().unwrap().download_mbps, res.download_mbps);
    ///
    /// clear_last_result_for_test();
    /// assert!(get_last_result().is_none());
    /// ```
    #[tokio::test]
    async fn test_set_get_clear_last_result_for_test() {
        clear_last_result_for_test();
        assert!(get_last_result().is_none());

        let res = dummy_result();
        set_last_result_for_test(res.clone());
        let cached = get_last_result().unwrap();
        assert_eq!(cached.download_mbps, res.download_mbps);

        clear_last_result_for_test();
        assert!(get_last_result().is_none());
    }

}
