[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SyncStatus

# Interface: SyncStatus

Defined in: [modules/bitview-client/index.js:1415](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1415)

## Properties

### blocksBehind

> **blocksBehind**: `number`

Defined in: [modules/bitview-client/index.js:1420](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1420)

Number of blocks behind the tip

***

### computedHeight

> **computedHeight**: `number`

Defined in: [modules/bitview-client/index.js:1417](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1417)

Height of the last computed block (series)

***

### indexedHeight

> **indexedHeight**: `number`

Defined in: [modules/bitview-client/index.js:1416](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1416)

Height of the last indexed block

***

### lastIndexedAt

> **lastIndexedAt**: `string`

Defined in: [modules/bitview-client/index.js:1421](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1421)

Human-readable timestamp of the last indexed block (ISO 8601)

***

### lastIndexedAtUnix

> **lastIndexedAtUnix**: `number`

Defined in: [modules/bitview-client/index.js:1422](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1422)

Unix timestamp of the last indexed block

***

### tipHeight

> **tipHeight**: `number`

Defined in: [modules/bitview-client/index.js:1418](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1418)

Height of the Bitcoin node's chain tip; `/health` reports `indexed_height` instead, without
  asking the node
