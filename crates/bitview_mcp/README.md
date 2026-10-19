# bitview_mcp

`bitview_mcp` is a stateless, read-only MCP adapter for the Bitview REST API. It
exposes the MCP-visible OpenAPI operations as generated tools and forwards
every tool call to the configured public REST origin as a `GET` request.

The official public endpoint is
[mcp.bitview.space](https://mcp.bitview.space/); it requires no authentication.

The server also serves an instance landing page, `/privacy`, `/terms`,
`/support`, and `/logo.png`. The icon is embedded in the binary and advertised
in MCP discovery metadata.

## Caching model

`bitview_mcp` does not cache API responses or retain MCP sessions. Point it at the
public Cloudflare-fronted REST origin so its upstream `GET` requests use the
existing Cloudflare cache:

```text
MCP client -> bitview_mcp -> Cloudflare-cached REST API -> Bitview server
```

Cloudflare does not need to cache the MCP endpoint. The MCP catalog TTL is
a client-side cache hint for the static discovery and tool-list
metadata.

## Run

From the workspace:

```sh
cargo run -p bitview_mcp -- \
  --api https://bitview.space \
  --url https://mcp.bitview.space/ \
  --name Bitview
```

For an installed binary, replace `cargo run -p bitview_mcp --` with `bitview_mcp`.

All three options are required:

- `--api` is the upstream Bitview REST origin.
- `--url` is this server's public MCP URL.
- `--name` is the human-readable instance name shown on the landing page and
  in MCP discovery metadata.

For local development:

```sh
bitview_mcp \
  --api http://127.0.0.1:3110 \
  --url http://127.0.0.1:3111/ \
  --name "Local Bitview"
```

The Streamable HTTP endpoint is `http://127.0.0.1:3111/` by default. If that
port is unavailable, the server tries each port through `3211`. The server
supports MCP protocol version `2026-07-28`.

Both URLs must be absolute HTTP(S) origins without a path, query, or
credentials. The public URL is normalized with a trailing slash.

## Generated catalog

`cargo bindgen` generates the tool catalog (`generated/manifest.json`, embedded at compile
time) and the MCP Registry `server.json`; see
[`bitview_bindgen`](../bitview_bindgen/README.md#regenerating).
