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
BIND_ADDRESS=<ADDRESS> BIND_PORT=<PORT> target/release/speedtest_statuspage
```

## Development

```bash
cargo build
cargo test
```