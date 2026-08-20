# bitview_mcp

`bitview_mcp` is a thin, stateless, read-only MCP adapter for the Bitview REST API. It
exposes the MCP-visible OpenAPI operations as generated tools and forwards
every tool call to the configured public REST origin as a `GET` request.

The official public endpoint is
[mcp.bitview.space](https://mcp.bitview.space/). It is stateless, read-only, and
requires no authentication.

Each deployment also serves instance-specific pages at `/privacy`, `/terms`,
and `/support`, plus its production icon at `/logo.png`. The landing page and
all three pages are rendered from the same embedded HTML file. The icon is
embedded in the server binary and advertised in MCP discovery metadata.

## Caching model

`bitview_mcp` does not cache API responses or retain MCP sessions. Point it at the
public Cloudflare-fronted REST origin so its upstream `GET` requests use the
existing Cloudflare cache:

```text
MCP client -> bitview_mcp -> Cloudflare-cached REST API -> Bitview server
```

Cloudflare does not need to cache the MCP endpoint. The MCP catalog TTL is
only a standard client-side cache hint for the static discovery and tool-list
metadata.

## Run

From the workspace:

```sh
cargo run --bin bitview_mcp -- \
  --api https://bitview.space \
  --url https://mcp.bitview.space/ \
  --name Bitview
```

Or run an installed binary:

```sh
bitview_mcp \
  --api https://api.example.com \
  --url https://mcp.example.com/ \
  --name "Example Node"
```

All three options are required:

- `--api` is the upstream Bitview REST origin. A bare host tries HTTPS first and
  retries over HTTP only if HTTPS fails at the transport layer.
- `--url` is this server's public MCP URL. It must be an absolute HTTP(S)
  origin.
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

Both URLs must be origins without a path, query, or credentials. The public URL
is normalized with a trailing slash.

## Generated catalog

The tool catalog is generated from the canonical OpenAPI document by
`bitview_bindgen` and embedded in the binary at compile time from
`generated/manifest.json`. Do not edit the generated manifest by hand; after
changing the API, run:

```sh
cargo run -p bitviewd --example bindgen --features bindgen
```
