---
name: bitview
description: "Fetches live and historical Bitcoin data from Bitview, read from a full node: on-chain metrics (MVRV, realized price, SOPR, holder cohorts, supply by age), blocks, transactions, addresses, mempool and fees, mining and difficulty, and the BTC/USD price computed from the chain. Use when a question needs Bitcoin on-chain, market, mempool, fee, mining or holder-cohort data."
---

# Bitview

Bitview is a Bitcoin data API read from a full node. Free and read-only, with no API key. Start from https://bitview.space/llms.txt; https://bitview.space/llms-full.txt is the full API reference. Call the REST API at https://bitview.space directly.

For a series:

1. `GET /api/series/search?q=<q>`: search by meaning with plain words; it returns series ids.
2. `GET /api/series/{series}`: read a returned id's description, supported indexes, value type, unit (what the value type measures), and the indexes whose values can be null (missing or undefined).
3. `GET /api/series/{series}/{index}`, `GET /api/series/{series}/{index}/latest`, `GET /api/series/bulk?series=<series>&index=<index>`: pick one of those indexes and fetch its values (`start` inclusive, `end` exclusive), the latest value alone, or several comma-separated series at once.

Never invent a series id: only ids the API returned exist.
