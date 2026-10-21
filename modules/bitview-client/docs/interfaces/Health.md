[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / Health

# Interface: Health

Defined in: [modules/bitview-client/index.js:599](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L599)

## Properties

### blocksBehind

> **blocksBehind**: `number`

Defined in: [modules/bitview-client/index.js:610](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L610)

Number of blocks behind the tip

***

### computedHeight

> **computedHeight**: `number`

Defined in: [modules/bitview-client/index.js:607](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L607)

Height of the last computed block (series)

***

### indexedHeight

> **indexedHeight**: `number`

Defined in: [modules/bitview-client/index.js:606](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L606)

Height of the last indexed block

***

### lastIndexedAt

> **lastIndexedAt**: `string`

Defined in: [modules/bitview-client/index.js:611](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L611)

Human-readable timestamp of the last indexed block (ISO 8601)

***

### lastIndexedAtUnix

> **lastIndexedAtUnix**: `number`

Defined in: [modules/bitview-client/index.js:612](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L612)

Unix timestamp of the last indexed block

***

### service

> **service**: `"brk"`

Defined in: [modules/bitview-client/index.js:601](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L601)

***

### startedAt

> **startedAt**: `string`

Defined in: [modules/bitview-client/index.js:604](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L604)

Server start time (ISO 8601)

***

### status

> **status**: `"healthy"`

Defined in: [modules/bitview-client/index.js:600](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L600)

***

### timestamp

> **timestamp**: `string`

Defined in: [modules/bitview-client/index.js:603](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L603)

Current server time (ISO 8601)

***

### tipHeight

> **tipHeight**: `number`

Defined in: [modules/bitview-client/index.js:608](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L608)

Height of the Bitcoin node's chain tip; `/health` reports `indexed_height` instead, without
asking the node

***

### uptimeSeconds

> **uptimeSeconds**: `number`

Defined in: [modules/bitview-client/index.js:605](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L605)

Uptime in seconds

***

### version

> **version**: `string`

Defined in: [modules/bitview-client/index.js:602](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L602)
