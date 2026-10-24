[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SyncStatus

# Interface: SyncStatus

Defined in: [modules/bitview-client/index.js:1331](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1331)

## Properties

### blocksBehind

> **blocksBehind**: `number`

Defined in: [modules/bitview-client/index.js:1336](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1336)

Number of blocks behind the tip

***

### computedHeight

> **computedHeight**: `number`

Defined in: [modules/bitview-client/index.js:1333](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1333)

Height of the last computed block (series)

***

### indexedHeight

> **indexedHeight**: `number`

Defined in: [modules/bitview-client/index.js:1332](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1332)

Height of the last indexed block

***

### lastIndexedAt

> **lastIndexedAt**: `string`

Defined in: [modules/bitview-client/index.js:1337](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1337)

Human-readable timestamp of the last indexed block (ISO 8601)

***

### lastIndexedAtUnix

> **lastIndexedAtUnix**: `number`

Defined in: [modules/bitview-client/index.js:1338](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1338)

Unix timestamp of the last indexed block

***

### tipHeight

> **tipHeight**: `number`

Defined in: [modules/bitview-client/index.js:1334](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1334)

Height of the Bitcoin node's chain tip; `/health` reports `indexed_height` instead, without
asking the node
