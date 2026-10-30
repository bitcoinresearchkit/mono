[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SyncStatus

# Interface: SyncStatus

Defined in: modules/bitview-client/index.js:1407

## Properties

### blocksBehind

> **blocksBehind**: `number`

Defined in: modules/bitview-client/index.js:1412

Number of blocks behind the tip

***

### computedHeight

> **computedHeight**: `number`

Defined in: modules/bitview-client/index.js:1409

Height of the last computed block (series)

***

### indexedHeight

> **indexedHeight**: `number`

Defined in: modules/bitview-client/index.js:1408

Height of the last indexed block

***

### lastIndexedAt

> **lastIndexedAt**: `string`

Defined in: modules/bitview-client/index.js:1413

Human-readable timestamp of the last indexed block (ISO 8601)

***

### lastIndexedAtUnix

> **lastIndexedAtUnix**: `number`

Defined in: modules/bitview-client/index.js:1414

Unix timestamp of the last indexed block

***

### tipHeight

> **tipHeight**: `number`

Defined in: modules/bitview-client/index.js:1410

Height of the Bitcoin node's chain tip; `/health` reports `indexed_height` instead, without
asking the node
