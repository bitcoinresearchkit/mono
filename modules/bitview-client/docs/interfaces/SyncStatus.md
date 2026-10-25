[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SyncStatus

# Interface: SyncStatus

Defined in: [modules/bitview-client/index.js:1381](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1381)

## Properties

### blocksBehind

> **blocksBehind**: `number`

Defined in: [modules/bitview-client/index.js:1386](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1386)

Number of blocks behind the tip

***

### computedHeight

> **computedHeight**: `number`

Defined in: [modules/bitview-client/index.js:1383](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1383)

Height of the last computed block (series)

***

### indexedHeight

> **indexedHeight**: `number`

Defined in: [modules/bitview-client/index.js:1382](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1382)

Height of the last indexed block

***

### lastIndexedAt

> **lastIndexedAt**: `string`

Defined in: [modules/bitview-client/index.js:1387](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1387)

Human-readable timestamp of the last indexed block (ISO 8601)

***

### lastIndexedAtUnix

> **lastIndexedAtUnix**: `number`

Defined in: [modules/bitview-client/index.js:1388](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1388)

Unix timestamp of the last indexed block

***

### tipHeight

> **tipHeight**: `number`

Defined in: [modules/bitview-client/index.js:1384](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1384)

Height of the Bitcoin node's chain tip; `/health` reports `indexed_height` instead, without
asking the node
