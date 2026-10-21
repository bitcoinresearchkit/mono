[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SyncStatus

# Interface: SyncStatus

Defined in: [modules/bitview-client/index.js:1212](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1212)

## Properties

### blocksBehind

> **blocksBehind**: `number`

Defined in: [modules/bitview-client/index.js:1217](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1217)

Number of blocks behind the tip

***

### computedHeight

> **computedHeight**: `number`

Defined in: [modules/bitview-client/index.js:1214](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1214)

Height of the last computed block (series)

***

### indexedHeight

> **indexedHeight**: `number`

Defined in: [modules/bitview-client/index.js:1213](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1213)

Height of the last indexed block

***

### lastIndexedAt

> **lastIndexedAt**: `string`

Defined in: [modules/bitview-client/index.js:1218](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1218)

Human-readable timestamp of the last indexed block (ISO 8601)

***

### lastIndexedAtUnix

> **lastIndexedAtUnix**: `number`

Defined in: [modules/bitview-client/index.js:1219](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1219)

Unix timestamp of the last indexed block

***

### tipHeight

> **tipHeight**: `number`

Defined in: [modules/bitview-client/index.js:1215](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1215)

Height of the Bitcoin node's chain tip; `/health` reports `indexed_height` instead, without
asking the node
