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

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ClientInfo {
    country: String,
    ip: String,
    isp: String,
    ispdlavg: String,
    isprating: String,
    ispulavg: String,
    lat: String,
    loggedin: String,
    lon: String,
    rating: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ServerInfo {
    cc: String,
    country: String,
    d: f64,
    host: String,
    id: String,
    lat: String,
    latency: f64,
    lon: String,
    name: String,
    sponsor: String,
    url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SpeedTestResponse {
    pub(crate) bytes_received: usize,
    pub(crate) bytes_sent: usize,
    pub(crate) client: ClientInfo,
    pub(crate) download: f64, // bits per second
    pub(crate) ping: f64,
    pub(crate) server: ServerInfo,
    pub(crate) share: Option<serde_json::Value>, // can be null or some data
    pub(crate) timestamp: String,
    pub(crate) upload: f64, // bits per second
}

#[derive(Serialize)]
pub(crate) struct SpeedTestResult {
    pub(crate) bytes_received: usize,
    pub(crate) bytes_sent: usize,
    pub(crate) download_bps: f64,
    pub(crate) upload_bps: f64,
    pub(crate) download_mbps: f64,
    pub(crate) upload_mbps: f64,
    pub(crate) ping_ms: f64,
    pub(crate) client: ClientInfo,
    pub(crate) server: ServerInfo,
    pub(crate) share: Option<serde_json::Value>,
    pub(crate) timestamp: String,
}