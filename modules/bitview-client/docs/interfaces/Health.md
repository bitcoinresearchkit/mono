[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / Health

# Interface: Health

Defined in: [modules/bitview-client/index.js:666](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L666)

## Properties

### blocksBehind

> **blocksBehind**: `number`

Defined in: [modules/bitview-client/index.js:677](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L677)

Number of blocks behind the tip

***

### computedHeight

> **computedHeight**: `number`

Defined in: [modules/bitview-client/index.js:674](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L674)

Height of the last computed block (series)

***

### indexedHeight

> **indexedHeight**: `number`

Defined in: [modules/bitview-client/index.js:673](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L673)

Height of the last indexed block

***

### lastIndexedAt

> **lastIndexedAt**: `string`

Defined in: [modules/bitview-client/index.js:678](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L678)

Human-readable timestamp of the last indexed block (ISO 8601)

***

### lastIndexedAtUnix

> **lastIndexedAtUnix**: `number`

Defined in: [modules/bitview-client/index.js:679](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L679)

Unix timestamp of the last indexed block

***

### service

> **service**: `"brk"`

Defined in: [modules/bitview-client/index.js:668](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L668)

***

### startedAt

> **startedAt**: `string`

Defined in: [modules/bitview-client/index.js:671](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L671)

Server start time (ISO 8601)

***

### status

> **status**: `"healthy"`

Defined in: [modules/bitview-client/index.js:667](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L667)

***

### timestamp

> **timestamp**: `string`

Defined in: [modules/bitview-client/index.js:670](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L670)

Current server time (ISO 8601)

***

### tipHeight

> **tipHeight**: `number`

Defined in: [modules/bitview-client/index.js:675](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L675)

Height of the Bitcoin node's chain tip; `/health` reports `indexed_height` instead, without
asking the node

***

### uptimeSeconds

> **uptimeSeconds**: `number`

Defined in: [modules/bitview-client/index.js:672](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L672)

Uptime in seconds

***

### version

> **version**: `string`

Defined in: [modules/bitview-client/index.js:669](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L669)
