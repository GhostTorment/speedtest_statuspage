# cd into the directory where this script is located
# shellcheck disable=SC2164
cd "$(dirname "$0")"

cargo clean
cargo build --profile release

cp .env package/etc/speedtest-statuspage/.env
cp target/release/speedtest-statuspage package/usr/local/bin/speedtest-statuspage