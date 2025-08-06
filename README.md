# speedtest_statuspage

This is a small utility application that will perform a speedtest on a GET request to the /speed endpoint, and return the results as json.

## Host Dependencies

`speedtest-cli >= 2.1.3-2`


## Disclaimer

This project is not affiliated with, endorsed by, or sponsored by Ookla. (Ookla®).
All trademarks and copyrights belong to their respective owners.

## Usage

```bash
cargo build --profile release
BIND_ADDRESS=<ADDRESS> BIND_PORT=<PORT> MIN_INTERVAL_MINUTES=<MINUTES> target/release/speedtest_statuspage
```

```dotenv
BIND_ADDRESS=0.0.0.0 # The address that the server will bind to
BIND_PORT=8080 # The port that the server will bind to
MIN_INTERVAL_MINUTES=60 # The interval for which a cache will be returned
```

## Development

```bash
cargo build
cargo test
```