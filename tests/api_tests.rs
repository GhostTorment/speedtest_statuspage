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

use actix_web::{test, http, App};
use serial_test::serial;
use speedtest_statuspage::*;

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