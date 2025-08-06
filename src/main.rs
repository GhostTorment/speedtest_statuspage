mod models;

use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use dotenvy;
use once_cell::sync::Lazy;
use std::env;
use std::process::Stdio;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::{process::Command, time};

use models::{SpeedTestResponse, SpeedTestResult};

static LAST_RESULT: Lazy<Mutex<Option<(SpeedTestResult, Instant)>>> = Lazy::new(|| Mutex::new(None));

#[get("/speed")]
async fn speedtest() -> impl Responder {
    let cache = LAST_RESULT.lock().unwrap();
    if let Some((cached_result, _timestamp)) = &*cache {
        HttpResponse::Ok().json(cached_result)
    } else {
        HttpResponse::ServiceUnavailable().body("Speedtest result not available yet.")
    }
}

fn min_frequency_duration() -> Duration {
    let minutes = env::var("INTERVAL_MINUTES")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(10); // default: 10 minutes
    Duration::from_secs(minutes * 60)
}

async fn run_speedtest_and_cache() {
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

                    let mut cache = LAST_RESULT.lock().unwrap();
                    *cache = Some((result.clone(), Instant::now()));
                    println!("Speedtest updated at {}", result.timestamp);
                }
                Err(e) => eprintln!("Failed to parse speedtest-cli JSON: {}", e),
            }
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("speedtest-cli failed: {}", stderr);
        }
        Err(e) => eprintln!("Failed to run speedtest-cli: {}", e),
    }
}

async fn spawn_speedtest_scheduler() {
    let interval = min_frequency_duration();

    // Run one immediately on startup
    run_speedtest_and_cache().await;

    let mut ticker = time::interval(interval);
    loop {
        ticker.tick().await;
        run_speedtest_and_cache().await;
    }
}

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