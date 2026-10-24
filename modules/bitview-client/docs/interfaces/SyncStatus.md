[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SyncStatus

# Interface: SyncStatus

Defined in: [modules/bitview-client/index.js:1313](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1313)

## Properties

### blocksBehind

> **blocksBehind**: `number`

Defined in: [modules/bitview-client/index.js:1318](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1318)

Number of blocks behind the tip

***

### computedHeight

> **computedHeight**: `number`

Defined in: [modules/bitview-client/index.js:1315](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1315)

Height of the last computed block (series)

***

### indexedHeight

> **indexedHeight**: `number`

Defined in: [modules/bitview-client/index.js:1314](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1314)

Height of the last indexed block

***

### lastIndexedAt

> **lastIndexedAt**: `string`

Defined in: [modules/bitview-client/index.js:1319](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1319)

Human-readable timestamp of the last indexed block (ISO 8601)

***

### lastIndexedAtUnix

> **lastIndexedAtUnix**: `number`

Defined in: [modules/bitview-client/index.js:1320](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1320)

Unix timestamp of the last indexed block

***

### tipHeight

> **tipHeight**: `number`

Defined in: [modules/bitview-client/index.js:1316](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1316)

Height of the Bitcoin node's chain tip; `/health` reports `indexed_height` instead, without
asking the node
