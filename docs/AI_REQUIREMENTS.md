# AI Product Requirements

> **Status:** Internal project requirements. This document is not a protocol
> specification, legal policy, or service-level agreement. Target requirements
> are not claims that every capability is already implemented.

This document guides AI-related work across Bitcoin Research Kit and Bitview. It
defines the project identity, current technical boundaries, target experience,
and criteria used to decide whether a capability is ready for AI use.

## Identity

- **Bitcoin Research Kit (BRK)** is the open-source project: the software, data
  engine, API specification, MCP implementation, and client libraries.
- **Bitview** is the official public BRK instance at
  [bitview.space](https://bitview.space).
- **Bitview MCP** is the official public BRK MCP instance at
  [mcp.bitview.space](https://mcp.bitview.space).
- Public material should describe Bitview as **powered by Bitcoin Research
  Kit**.

Project and protocol requirements belong to BRK. Requirements concerning the
official hosted service belong to Bitview.

## Current technical boundaries

These properties describe the current implementation and must remain explicit
in AI-facing documentation.

### REST API

- Bitview's REST API is public and does not require an account or API key.
- Its permissive CORS policy allows static browser applications to call it
  directly.
- `POST /api/tx` can relay a caller-provided raw transaction to the Bitcoin
  network through Bitcoin Core.
- BRK does not hold private keys, sign transactions, control wallets, or trade.

### MCP

- Bitview MCP uses Streamable HTTP and does not require authentication.
- It has no request rate limit or usage quota.
- It supports MCP `2026-07-28` only and does not provide a legacy handshake.
- Requests must use the modern stateless protocol metadata.
- It is stateless: requests do not depend on an MCP session.
- Its tools are generated only from non-deprecated REST `GET` operations.
- It does not expose transaction broadcasting and is therefore read-only.

## Product promise

Bitcoin Research Kit is open-source Bitcoin data infrastructure for humans,
applications, and AI. Bitview is its official free public instance, providing
unlimited and unauthenticated access without requiring an application backend.

An AI starting from one Bitview documentation entry point should be able to
discover, understand, and retrieve Bitcoin data, then use it to create a working
browser application.

## Target requirements

### Free public access

Bitview must require no account, API key, subscription, usage quota, or rate
limit.

### Browser-first access

A static browser application must be able to use Bitview directly without a
secret, authenticated proxy, or application server.

### Understandable data

AI-facing metric metadata must communicate the metric's meaning, unit, value
type, available indexes, methodology, freshness, provenance, and relevant
caveats. An AI must not need to infer these from an internal type or series
name.

### Stable interfaces

Public interfaces must provide stable operation and field names. A breaking
change requires a documented replacement and deprecation path before removal.

### Vendor-neutral access

The core data contract must not depend on a specific AI vendor or model. BRK
must expose it through standard REST and OpenAPI interfaces and Streamable HTTP
MCP. Maintained JavaScript, Python, and Rust clients should derive from the same
contract.

### Explicit behavior

AI-facing documentation and responses must make data freshness, units,
missing-data behavior, experimental status, deprecations, and failures clear.
An AI must not need to guess whether a value is current or why a request failed.

### One canonical source

OpenAPI, MCP tools, client libraries, AI documentation, and examples must derive
their descriptions and schemas from the same source wherever practical. Tests
must detect contradictions between generated surfaces.

## North-star workflow

The canonical entry point is `https://mcp.bitview.space/`. Starting only from
that page and its linked BRK resources, a supported coding AI should be able to
produce a working, deployable, single-file Bitcoin application in under ten
minutes, without manual API debugging.

The initial reference applications are:

1. A historical Bitcoin metric chart.
2. A live network and mempool dashboard.
3. A block, transaction, and address explorer.

Each application must:

- Run directly in a browser.
- Require no secret or backend.
- Handle loading, empty, and failure states.
- Identify Bitview as its data source.
- Show when its data was last updated.

The models, prompts, expected results, and evaluation environment must be
recorded so results can be repeated and compared over time.

## Definition of AI-ready

A BRK or Bitview capability is AI-ready when a supported AI can:

1. Discover it from the canonical documentation.
2. Select the correct operation and parameters.
3. Understand the returned values and units.
4. Handle documented errors without inventing data or identifiers.
5. Use it in a working application without private project knowledge.

Readiness must be demonstrated with generated documentation, contract tests,
reference applications, and repeatable AI evaluations.
