# speedtest-statuspage

A utility application to serve speedtest results over an HTTP endpoint.

---

## Disclaimer

This project is not affiliated with, endorsed by, or sponsored by Ookla (Ookla®).  
All trademarks and copyrights belong to their respective owners.

---

## Overview

`speedtest-statuspage` runs periodic speedtests using the `speedtest-cli` binary, caches the latest results, and exposes them via an HTTP JSON API endpoint (`/speed`).  
The service can be configured via environment variables and is built using Rust with Actix-web and Tokio for async task scheduling.

---

## Features

- Periodically runs `speedtest-cli` every N minutes (default: 60).
- Caches the last successful speedtest result in memory.
- Exposes `/speed` HTTP GET endpoint returning the latest cached speedtest result as JSON.
- Returns HTTP 503 if no cached speedtest result is available yet.
- Configurable bind address, port, and speedtest interval via environment variables.

---

## Environment Variables

| Variable         | Description                              | Default   |  
|------------------|------------------------------------------|-----------|  
| `BIND_ADDRESS`   | IP address to bind the HTTP server       | `127.0.0.1` |  
| `BIND_PORT`      | Port for the HTTP server                  | `8080`    |  
| `INTERVAL_MINUTES` | Interval in minutes between speedtests | `60`      |

---

## Usage

1. Ensure `speedtest-cli` is installed and available in your system `PATH`.

2. Set environment variables as needed, for example:

    ```shell
    export BIND_ADDRESS=0.0.0.0
    export BIND_PORT=8080
    export INTERVAL_MINUTES=5
    ```

3. Run the application:

    ```shell
    cargo run --release
    ```

4. Access speedtest results at:

    ```
    http://<BIND_ADDRESS>:<BIND_PORT>/speed
    ```

   The endpoint returns JSON with the latest speedtest data or HTTP 503 if no results are available yet.

---

## Example Response

```json
{
  "bytes_received": 12345678,
  "bytes_sent": 8765432,
  "download_bps": 50000000.0,
  "upload_bps": 10000000.0,
  "download_mbps": 50.0,
  "upload_mbps": 10.0,
  "ping_ms": 15.3,
  "client": { /* client info */ }
}
