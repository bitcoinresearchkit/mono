[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SyncStatus

# Interface: SyncStatus

Defined in: [modules/bitview-client/index.js:1279](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1279)

## Properties

### blocksBehind

> **blocksBehind**: `number`

Defined in: [modules/bitview-client/index.js:1284](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1284)

Number of blocks behind the tip

***

### computedHeight

> **computedHeight**: `number`

Defined in: [modules/bitview-client/index.js:1281](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1281)

Height of the last computed block (series)

***

### indexedHeight

> **indexedHeight**: `number`

Defined in: [modules/bitview-client/index.js:1280](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1280)

Height of the last indexed block

***

### lastIndexedAt

> **lastIndexedAt**: `string`

Defined in: [modules/bitview-client/index.js:1285](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1285)

Human-readable timestamp of the last indexed block (ISO 8601)

***

### lastIndexedAtUnix

> **lastIndexedAtUnix**: `number`

Defined in: [modules/bitview-client/index.js:1286](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1286)

Unix timestamp of the last indexed block

***

### tipHeight

> **tipHeight**: `number`

Defined in: [modules/bitview-client/index.js:1282](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1282)

Height of the Bitcoin node's chain tip; `/health` reports `indexed_height` instead, without
asking the node
