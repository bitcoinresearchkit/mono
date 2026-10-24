[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SyncStatus

# Interface: SyncStatus

Defined in: [modules/bitview-client/index.js:1302](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1302)

## Properties

### blocksBehind

> **blocksBehind**: `number`

Defined in: [modules/bitview-client/index.js:1307](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1307)

Number of blocks behind the tip

***

### computedHeight

> **computedHeight**: `number`

Defined in: [modules/bitview-client/index.js:1304](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1304)

Height of the last computed block (series)

***

### indexedHeight

> **indexedHeight**: `number`

Defined in: [modules/bitview-client/index.js:1303](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1303)

Height of the last indexed block

***

### lastIndexedAt

> **lastIndexedAt**: `string`

Defined in: [modules/bitview-client/index.js:1308](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1308)

Human-readable timestamp of the last indexed block (ISO 8601)

***

### lastIndexedAtUnix

> **lastIndexedAtUnix**: `number`

Defined in: [modules/bitview-client/index.js:1309](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1309)

Unix timestamp of the last indexed block

***

### tipHeight

> **tipHeight**: `number`

Defined in: [modules/bitview-client/index.js:1305](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1305)

Height of the Bitcoin node's chain tip; `/health` reports `indexed_height` instead, without
asking the node
