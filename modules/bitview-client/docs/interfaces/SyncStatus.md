[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SyncStatus

# Interface: SyncStatus

Defined in: [modules/bitview-client/index.js:1282](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1282)

## Properties

### blocksBehind

> **blocksBehind**: `number`

Defined in: [modules/bitview-client/index.js:1287](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1287)

Number of blocks behind the tip

***

### computedHeight

> **computedHeight**: `number`

Defined in: [modules/bitview-client/index.js:1284](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1284)

Height of the last computed block (series)

***

### indexedHeight

> **indexedHeight**: `number`

Defined in: [modules/bitview-client/index.js:1283](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1283)

Height of the last indexed block

***

### lastIndexedAt

> **lastIndexedAt**: `string`

Defined in: [modules/bitview-client/index.js:1288](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1288)

Human-readable timestamp of the last indexed block (ISO 8601)

***

### lastIndexedAtUnix

> **lastIndexedAtUnix**: `number`

Defined in: [modules/bitview-client/index.js:1289](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1289)

Unix timestamp of the last indexed block

***

### tipHeight

> **tipHeight**: `number`

Defined in: [modules/bitview-client/index.js:1285](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1285)

Height of the Bitcoin node's chain tip; `/health` reports `indexed_height` instead, without
  asking the node
