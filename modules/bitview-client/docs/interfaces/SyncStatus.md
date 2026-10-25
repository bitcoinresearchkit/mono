[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SyncStatus

# Interface: SyncStatus

Defined in: [modules/bitview-client/index.js:1403](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1403)

## Properties

### blocksBehind

> **blocksBehind**: `number`

Defined in: [modules/bitview-client/index.js:1408](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1408)

Number of blocks behind the tip

***

### computedHeight

> **computedHeight**: `number`

Defined in: [modules/bitview-client/index.js:1405](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1405)

Height of the last computed block (series)

***

### indexedHeight

> **indexedHeight**: `number`

Defined in: [modules/bitview-client/index.js:1404](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1404)

Height of the last indexed block

***

### lastIndexedAt

> **lastIndexedAt**: `string`

Defined in: [modules/bitview-client/index.js:1409](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1409)

Human-readable timestamp of the last indexed block (ISO 8601)

***

### lastIndexedAtUnix

> **lastIndexedAtUnix**: `number`

Defined in: [modules/bitview-client/index.js:1410](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1410)

Unix timestamp of the last indexed block

***

### tipHeight

> **tipHeight**: `number`

Defined in: [modules/bitview-client/index.js:1406](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1406)

Height of the Bitcoin node's chain tip; `/health` reports `indexed_height` instead, without
asking the node
