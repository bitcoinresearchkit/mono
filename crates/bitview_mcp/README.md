# bitview_mcp

`bitview_mcp` is a stateless, read-only MCP adapter for the Bitview REST API. It
exposes the MCP-visible OpenAPI operations as generated tools and forwards
every tool call to the configured public REST origin as a `GET` request.

Bitview is a free, open-source Bitcoin data server: on-chain metrics, blocks, mempool and mining data, with charts and an API, at [bitview.space](https://bitview.space) or beside your own Bitcoin Core node.

The official public endpoint is
[mcp.bitview.space](https://mcp.bitview.space/); it requires no authentication.

The server also serves an instance landing page, `/logo.png`, `/privacy`,
`/terms` and `/support` (redirects to the REST origin's privacy, terms and
support pages), and its discovery documents, an MCP Server Card at
`/server-card` and an AI Catalog at `/.well-known/ai-catalog.json`. The icon is
embedded in the binary and advertised in MCP discovery metadata.

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
supports MCP protocol versions `2025-06-18` through `2026-07-28`, without
sessions at any of them. Versionless legacy requests are accepted without retaining
a session or inferring a version from an earlier initialization.

Both URLs must be absolute HTTP(S) origins without a path, query, or
credentials. The public URL is normalized with a trailing slash.

## Generated catalog

`cargo bindgen` generates the tool catalog (`generated/manifest.json`, embedded at compile
time) and the MCP Registry `server.json`; see
[`bitview_bindgen`](../bitview_bindgen/README.md#regenerating).
