# bitview-cli

Generated command-line client for the [Bitview](https://bitview.space) API.
Every non-deprecated OpenAPI operation becomes a command derived from the
server specification by `bitview_bindgen`.

## Run

Install it from the BRK workspace:

```bash
cargo install --locked --path crates/bitview_cli
```

Or run it without installing:

```bash
cargo run -p bitview_cli -- get-block-by-height 800000
```

The default server is `http://localhost:3110`. Point the CLI at the public
instance with `--url` or `BITVIEW_URL`:

```bash
bitview-cli --url https://bitview.space get-block-by-height 800000
BITVIEW_URL=https://bitview.space bitview-cli get-health
```

Path parameters are positional and query parameters are named flags. Repeat an
array-valued query flag to send multiple values:

```bash
bitview-cli get-series-data realized_price day1 --start 6000 --end 6100
bitview-cli get-series realized_price day1 --format csv
bitview-cli post-tx --body 0200000001...
```

Use `--pretty` to format JSON responses. Command help includes the full OpenAPI
description and works before or after the command name:

```bash
bitview-cli --pretty get-health
bitview-cli help
bitview-cli help get-series-data
bitview-cli get-series-data help
```

## Regenerate

```bash
cargo run -p bitviewd --bin bitview-bindgen --features bindgen
```

`src/generated.rs` is generated and must not be edited by hand.

## License

MIT
