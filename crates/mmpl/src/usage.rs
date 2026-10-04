// Raw string contains `{`/`}` literals (JSON), so it can't be the
// format string of `print!`. Pass via positional arg.
#[allow(clippy::print_literal)]
pub fn print() {
    print!(
        "{}",
        r#"mmpl - stream Bitcoin mempool events as NDJSON

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

Events (one JSON object per line):
  Per-tx (one event per change):
    {"kind":"enter","t":..,"txid":..,"vsize":..,"fee":..,"rate":..,"first_seen":..}
    {"kind":"leave","t":..,"txid":..,"reason":"vanished","rate":..}
    {"kind":"leave","t":..,"txid":..,"reason":"replaced","by":..,"rate":..}

  Per-address (0 <-> 1+ live mempool txs):
    {"kind":"addr_enter","t":..,"addr":..}
    {"kind":"addr_leave","t":..,"addr":..}

  State changes (fires only when the value changed):
    {"kind":"tip","t":..,"hash":..,"height":..}        (new confirmed block)
    {"kind":"block","t":..,"hash":..,"added":[txid..],"removed":[txid..]}
                                                       (next-block template changed; first cycle
                                                        emits the full template as `added`)
    {"kind":"fees","t":..,"fastest":..,"half_hour":..,"hour":..,"economy":..,"minimum":..}

  Per-cycle heartbeat (always emitted):
    {"kind":"cycle","t":..,"added":N,"removed":N,"addr_enters":N,"addr_leaves":N,
                   "count":N,"vsize":N,"fee":N,"took_ms":N}

Examples:
  mmpl | jq -c 'select(.kind=="enter" and .rate>=50)'
  mmpl | jq -c 'select(.kind=="tip")'
  mmpl | grep -v '"kind":"cycle"'
  mmpl | jq -c 'select(.reason=="replaced")'
"#
    );
}
