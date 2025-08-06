cargo clean
cargo build --profile release

cp .env package/etc/speedtest-statuspage/.env
cp target/release/speedtest-statuspage package/usr/local/bin/speedtest-statuspage
