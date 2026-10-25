[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / Health

# Interface: Health

Defined in: [modules/bitview-client/index.js:671](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L671)

## Properties

### blocksBehind

> **blocksBehind**: `number`

Defined in: [modules/bitview-client/index.js:682](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L682)

Number of blocks behind the tip

***

### computedHeight

> **computedHeight**: `number`

Defined in: [modules/bitview-client/index.js:679](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L679)

Height of the last computed block (series)

***

### indexedHeight

> **indexedHeight**: `number`

Defined in: [modules/bitview-client/index.js:678](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L678)

Height of the last indexed block

***

### lastIndexedAt

> **lastIndexedAt**: `string`

Defined in: [modules/bitview-client/index.js:683](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L683)

Human-readable timestamp of the last indexed block (ISO 8601)

***

### lastIndexedAtUnix

> **lastIndexedAtUnix**: `number`

Defined in: [modules/bitview-client/index.js:684](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L684)

Unix timestamp of the last indexed block

***

### service

> **service**: `"brk"`

Defined in: [modules/bitview-client/index.js:673](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L673)

***

### startedAt

> **startedAt**: `string`

Defined in: [modules/bitview-client/index.js:676](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L676)

Server start time (ISO 8601)

***

### status

> **status**: `"healthy"`

Defined in: [modules/bitview-client/index.js:672](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L672)

***

### timestamp

> **timestamp**: `string`

Defined in: [modules/bitview-client/index.js:675](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L675)

Current server time (ISO 8601)

***

### tipHeight

> **tipHeight**: `number`

Defined in: [modules/bitview-client/index.js:680](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L680)

Height of the Bitcoin node's chain tip; `/health` reports `indexed_height` instead, without
asking the node

***

### uptimeSeconds

> **uptimeSeconds**: `number`

Defined in: [modules/bitview-client/index.js:677](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L677)

Uptime in seconds

***

### version

> **version**: `string`

Defined in: [modules/bitview-client/index.js:674](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L674)
