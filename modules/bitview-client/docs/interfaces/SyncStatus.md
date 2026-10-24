[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SyncStatus

# Interface: SyncStatus

Defined in: [modules/bitview-client/index.js:1307](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1307)

## Properties

### blocksBehind

> **blocksBehind**: `number`

Defined in: [modules/bitview-client/index.js:1312](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1312)

Number of blocks behind the tip

***

### computedHeight

> **computedHeight**: `number`

Defined in: [modules/bitview-client/index.js:1309](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1309)

Height of the last computed block (series)

***

### indexedHeight

> **indexedHeight**: `number`

Defined in: [modules/bitview-client/index.js:1308](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1308)

Height of the last indexed block

***

### lastIndexedAt

> **lastIndexedAt**: `string`

Defined in: [modules/bitview-client/index.js:1313](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1313)

Human-readable timestamp of the last indexed block (ISO 8601)

***

### lastIndexedAtUnix

> **lastIndexedAtUnix**: `number`

Defined in: [modules/bitview-client/index.js:1314](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1314)

Unix timestamp of the last indexed block

***

### tipHeight

> **tipHeight**: `number`

Defined in: [modules/bitview-client/index.js:1310](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1310)

Height of the Bitcoin node's chain tip; `/health` reports `indexed_height` instead, without
asking the node
