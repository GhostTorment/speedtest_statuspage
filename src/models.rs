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

/// Information about the client running the speedtest.
///
/// This struct represents client-related metadata returned from the speedtest API,
/// such as ISP details, location, and rating metrics.
///
/// # Examples
///
/// ```
/// use speedtest_statuspage::models::ClientInfo;
///
/// let client = ClientInfo {
///     country: "UK".to_string(),
///     ip: "192.0.2.1".to_string(),
///     isp: "Example ISP".to_string(),
///     ispdlavg: "50".to_string(),
///     isprating: "4".to_string(),
///     ispulavg: "10".to_string(),
///     lat: "51.5074".to_string(),
///     loggedin: "false".to_string(),
///     lon: "-0.1278".to_string(),
///     rating: "4.5".to_string(),
/// };
/// assert_eq!(client.country, "UK");
/// assert_eq!(client.ip, "192.0.2.1");
/// ```
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub(crate) struct ClientInfo {
    /// Client's country code or name.
    pub(crate) country: String,

    /// Client's IP address.
    pub(crate) ip: String,

    /// Client's Internet Service Provider.
    pub(crate) isp: String,

    /// ISP download average speed as a string.
    pub(crate) ispdlavg: String,

    /// ISP rating as a string.
    pub(crate) isprating: String,

    /// ISP upload average speed as a string.
    pub(crate) ispulavg: String,

    /// Client's latitude coordinate as a string.
    pub(crate) lat: String,

    /// Whether the client is logged in (string "true"/"false").
    pub(crate) loggedin: String,

    /// Client's longitude coordinate as a string.
    pub(crate) lon: String,

    /// Client rating as a string.
    pub(crate) rating: String,
}

/// Information about the server used in the speedtest.
///
/// Represents metadata about the test server including location,
/// latency, host info, and sponsor details.
///
/// # Examples
///
/// ```
/// use speedtest_statuspage::models::ServerInfo;
///
/// let server = ServerInfo {
///     cc: "GB".to_string(),
///     country: "United Kingdom".to_string(),
///     d: 5.0,
///     host: "speedtest.example.com".to_string(),
///     id: "12345".to_string(),
///     lat: "51.5074".to_string(),
///     latency: 10.5,
///     lon: "-0.1278".to_string(),
///     name: "London Server".to_string(),
///     sponsor: "Example Sponsor".to_string(),
///     url: "http://speedtest.example.com".to_string(),
/// };
/// assert_eq!(server.cc, "GB");
/// assert_eq!(server.latency, 10.5);
/// ```
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub(crate) struct ServerInfo {
    /// Server country code.
    pub(crate) cc: String,

    /// Server country name.
    pub(crate) country: String,

    /// Distance from client to server in kilometers.
    pub(crate) d: f64,

    /// Server hostname.
    pub(crate) host: String,

    /// Server identifier.
    pub(crate) id: String,

    /// Server latitude.
    pub(crate) lat: String,

    /// Server latency in milliseconds.
    pub(crate) latency: f64,

    /// Server longitude.
    pub(crate) lon: String,

    /// Server name.
    pub(crate) name: String,

    /// Server sponsor or operator.
    pub(crate) sponsor: String,

    /// URL of the server.
    pub(crate) url: String,
}

/// The raw response from `speedtest-cli` JSON output.
///
/// This struct matches the JSON structure returned by the CLI tool
/// and is used internally to deserialize speedtest results.
///
/// # Notes
///
/// - Download and upload speeds are represented in bits per second.
/// - The `share` field can be `null` or contain JSON data.
///
/// # Examples
///
/// ```
/// use speedtest_statuspage::models::SpeedTestResponse;
/// use serde_json::from_str;
///
/// let json_data = r#"
/// {
///     "bytes_received": 12345678,
///     "bytes_sent": 87654321,
///     "client": {
///         "country": "UK",
///         "ip": "192.0.2.1",
///         "isp": "Example ISP",
///         "ispdlavg": "50",
///         "isprating": "4",
///         "ispulavg": "10",
///         "lat": "51.5074",
///         "loggedin": "false",
///         "lon": "-0.1278",
///         "rating": "4.5"
///     },
///     "download": 50000000.0,
///     "ping": 20.0,
///     "server": {
///         "cc": "GB",
///         "country": "United Kingdom",
///         "d": 5.0,
///         "host": "speedtest.example.com",
///         "id": "12345",
///         "lat": "51.5074",
///         "latency": 10.5,
///         "lon": "-0.1278",
///         "name": "London Server",
///         "sponsor": "Example Sponsor",
///         "url": "http://speedtest.example.com"
///     },
///     "share": null,
///     "timestamp": "2025-08-07T12:00:00Z",
///     "upload": 10000000.0
/// }
/// "#;
///
/// let parsed: SpeedTestResponse = from_str(json_data).unwrap();
/// assert_eq!(parsed.bytes_received, 12345678);
/// assert_eq!(parsed.client.isp, "Example ISP");
/// assert_eq!(parsed.download, 50000000.0);
/// assert_eq!(parsed.server.name, "London Server");
/// ```
#[derive(Serialize, Deserialize, Debug)]
pub struct SpeedTestResponse {
    /// Number of bytes received during the test.
    pub(crate) bytes_received: usize,

    /// Number of bytes sent during the test.
    pub(crate) bytes_sent: usize,

    /// Client information.
    pub(crate) client: ClientInfo,

    /// Download speed in bits per second.
    pub(crate) download: f64,

    /// Ping time in milliseconds.
    pub(crate) ping: f64,

    /// Server information.
    pub(crate) server: ServerInfo,

    /// Optional share data returned by the speedtest service.
    pub(crate) share: Option<serde_json::Value>,

    /// ISO8601 timestamp of the test.
    pub(crate) timestamp: String,

    /// Upload speed in bits per second.
    pub(crate) upload: f64,
}

/// A processed and cached speedtest result ready for API serving.
///
/// This struct stores speeds both in bits per second and megabits per second,
/// along with ping and metadata about client/server.
///
/// It is used to cache and respond with speedtest data efficiently.
///
/// # Examples
///
/// ```
/// use speedtest_statuspage::models::{SpeedTestResult, ClientInfo, ServerInfo};
///
/// let result = SpeedTestResult {
///     bytes_received: 12345678,
///     bytes_sent: 87654321,
///     download_bps: 50000000.0,
///     upload_bps: 10000000.0,
///     download_mbps: 50.0,
///     upload_mbps: 10.0,
///     ping_ms: 20.0,
///     client: ClientInfo {
///         country: "UK".to_string(),
///         ip: "192.0.2.1".to_string(),
///         isp: "Example ISP".to_string(),
///         ispdlavg: "50".to_string(),
///         isprating: "4".to_string(),
///         ispulavg: "10".to_string(),
///         lat: "51.5074".to_string(),
///         loggedin: "false".to_string(),
///         lon: "-0.1278".to_string(),
///         rating: "4.5".to_string(),
///     },
///     server: ServerInfo {
///         cc: "GB".to_string(),
///         country: "United Kingdom".to_string(),
///         d: 5.0,
///         host: "speedtest.example.com".to_string(),
///         id: "12345".to_string(),
///         lat: "51.5074".to_string(),
///         latency: 10.5,
///         lon: "-0.1278".to_string(),
///         name: "London Server".to_string(),
///         sponsor: "Example Sponsor".to_string(),
///         url: "http://speedtest.example.com".to_string(),
///     },
///     share: None,
///     timestamp: "2025-08-07T12:00:00Z".to_string(),
/// };
///
/// assert_eq!(result.download_mbps, 50.0);
/// assert_eq!(result.client.isp, "Example ISP");
/// ```
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub(crate) struct SpeedTestResult {
    /// Number of bytes received.
    pub(crate) bytes_received: usize,

    /// Number of bytes sent.
    pub(crate) bytes_sent: usize,

    /// Download speed in bits per second.
    pub(crate) download_bps: f64,

    /// Upload speed in bits per second.
    pub(crate) upload_bps: f64,

    /// Download speed in megabits per second.
    pub(crate) download_mbps: f64,

    /// Upload speed in megabits per second.
    pub(crate) upload_mbps: f64,

    /// Ping time in milliseconds.
    pub(crate) ping_ms: f64,

    /// Client information.
    pub(crate) client: ClientInfo,

    /// Server information.
    pub(crate) server: ServerInfo,

    /// Optional share data from the speedtest service.
    pub(crate) share: Option<serde_json::Value>,

    /// Timestamp of the speedtest.
    pub(crate) timestamp: String,
}
