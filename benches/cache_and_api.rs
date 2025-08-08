use criterion::{criterion_group, criterion_main, Criterion};
use serde_json;
use speedtest_statuspage::{clear_last_result_for_test, get_last_result, set_last_result_for_test, SpeedTestResult};

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
        timestamp: "2025-08-07T12:34:56Z".into(),
    }
}

fn bench_set_get_clear(c: &mut Criterion) {
    let res = dummy_result();
    c.bench_function("set_get_clear", |b| {
        b.iter(|| {
            set_last_result_for_test(res.clone());
            let _ = get_last_result();
            clear_last_result_for_test();
        })
    });
}

fn bench_serialize(c: &mut Criterion) {
    let res = dummy_result();
    c.bench_function("serialize_speedtest_result", |b| {
        b.iter(|| serde_json::to_string(&res).unwrap())
    });
}

fn bench_speed_endpoint_cached(c: &mut Criterion) {
    set_last_result_for_test(dummy_result());
    c.bench_function("speed_endpoint_cached", |b| {
        b.iter(|| {
            let _ = futures::executor::block_on(speedtest_statuspage::get_cached_speedtest_result()).unwrap();
        })
    });
}

criterion_group!(
    benches,
    bench_set_get_clear,
    bench_serialize,
    bench_speed_endpoint_cached
);
criterion_main!(benches);
