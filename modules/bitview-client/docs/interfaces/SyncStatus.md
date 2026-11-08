[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SyncStatus

# Interface: SyncStatus

Defined in: [modules/bitview-client/index.js:1359](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1359)

## Properties

### blocksBehind

> **blocksBehind**: `number`

Defined in: [modules/bitview-client/index.js:1364](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1364)

Number of blocks behind the tip

***

### computedHeight

> **computedHeight**: `number`

Defined in: [modules/bitview-client/index.js:1361](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1361)

Height of the last computed block (series)

***

### indexedHeight

> **indexedHeight**: `number`

Defined in: [modules/bitview-client/index.js:1360](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1360)

Height of the last indexed block

***

### lastIndexedAt

> **lastIndexedAt**: `string`

Defined in: [modules/bitview-client/index.js:1365](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1365)

Human-readable timestamp of the last indexed block (ISO 8601)

***

### lastIndexedAtUnix

> **lastIndexedAtUnix**: `number`

Defined in: [modules/bitview-client/index.js:1366](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1366)

Unix timestamp of the last indexed block

***

### tipHeight

> **tipHeight**: `number`

Defined in: [modules/bitview-client/index.js:1362](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1362)

Height of the Bitcoin node's chain tip; `/health` reports `indexed_height` instead, without
  asking the node
