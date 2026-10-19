pub fn print() {
    print!("{USAGE}");
}

const USAGE: &str = r#"mmpl - stream Bitcoin mempool events as NDJSON

Usage:
  mmpl [options]

Options:
  --bitcoindir <path>      Bitcoin data dir (default: platform-specific)
  --rpcconnect <host>      RPC host (default: localhost)
  --rpcport <port>         RPC port (default: 8332)
  --rpccookiefile <path>   Cookie file; relative paths resolve inside <bitcoindir> (default: <bitcoindir>/.cookie)
  --rpcuser <user>         RPC username (used with --rpcpassword)
  --rpcpassword <pass>     RPC password (when set, used instead of the cookie)
  -h, --help               Show this help

Events (one JSON object per line), from each published pool against the previous one:
  Per-tx (one event per change):
    {"kind":"enter","t":..,"txid":..,"vsize":..,"fee":..,"rate":..,"first_seen":..}
    {"kind":"leave","t":..,"txid":..,"reason":"vanished"}
    {"kind":"leave","t":..,"txid":..,"reason":"replaced","by":..}

  State (on the first cycle, then whenever the value changes):
    {"kind":"tip","t":..,"hash":..,"height":..}        (chain tip)
    {"kind":"block","t":..,"hash":..,"count":..,"vsize":..,"fee":..}
                                                       (next-block template)
    {"kind":"fees","t":..,"fastest":..,"half_hour":..,"hour":..,"economy":..,"minimum":..}

  Per-cycle heartbeat (every successful cycle):
    {"kind":"cycle","t":..,"added":N,"removed":N,"count":N,"vsize":N,"fee":N,"took_ms":N}

Examples:
  mmpl | jq -c 'select(.kind=="enter" and .rate>=50)'
  mmpl | jq -c 'select(.kind=="tip")'
  mmpl | grep -v '"kind":"cycle"'
  mmpl | jq -c 'select(.reason=="replaced")'
"#;
