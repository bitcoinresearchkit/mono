[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / Health

# Interface: Health

Defined in: [modules/bitview-client/index.js:656](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L656)

## Properties

### blocksBehind

> **blocksBehind**: `number`

Defined in: [modules/bitview-client/index.js:667](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L667)

Number of blocks behind the tip

***

### computedHeight

> **computedHeight**: `number`

Defined in: [modules/bitview-client/index.js:664](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L664)

Height of the last computed block (series)

***

### indexedHeight

> **indexedHeight**: `number`

Defined in: [modules/bitview-client/index.js:663](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L663)

Height of the last indexed block

***

### lastIndexedAt

> **lastIndexedAt**: `string`

Defined in: [modules/bitview-client/index.js:668](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L668)

Human-readable timestamp of the last indexed block (ISO 8601)

***

### lastIndexedAtUnix

> **lastIndexedAtUnix**: `number`

Defined in: [modules/bitview-client/index.js:669](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L669)

Unix timestamp of the last indexed block

***

### service

> **service**: `"brk"`

Defined in: [modules/bitview-client/index.js:658](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L658)

***

### startedAt

> **startedAt**: `string`

Defined in: [modules/bitview-client/index.js:661](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L661)

Server start time (ISO 8601)

***

### status

> **status**: `"healthy"`

Defined in: [modules/bitview-client/index.js:657](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L657)

***

### timestamp

> **timestamp**: `string`

Defined in: [modules/bitview-client/index.js:660](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L660)

Current server time (ISO 8601)

***

### tipHeight

> **tipHeight**: `number`

Defined in: [modules/bitview-client/index.js:665](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L665)

Height of the Bitcoin node's chain tip; `/health` reports `indexed_height` instead, without
asking the node

***

### uptimeSeconds

> **uptimeSeconds**: `number`

Defined in: [modules/bitview-client/index.js:662](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L662)

Uptime in seconds

***

### version

> **version**: `string`

Defined in: [modules/bitview-client/index.js:659](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L659)
