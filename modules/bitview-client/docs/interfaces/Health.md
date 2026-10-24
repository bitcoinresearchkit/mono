[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / Health

# Interface: Health

Defined in: [modules/bitview-client/index.js:641](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L641)

## Properties

### blocksBehind

> **blocksBehind**: `number`

Defined in: [modules/bitview-client/index.js:652](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L652)

Number of blocks behind the tip

***

### computedHeight

> **computedHeight**: `number`

Defined in: [modules/bitview-client/index.js:649](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L649)

Height of the last computed block (series)

***

### indexedHeight

> **indexedHeight**: `number`

Defined in: [modules/bitview-client/index.js:648](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L648)

Height of the last indexed block

***

### lastIndexedAt

> **lastIndexedAt**: `string`

Defined in: [modules/bitview-client/index.js:653](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L653)

Human-readable timestamp of the last indexed block (ISO 8601)

***

### lastIndexedAtUnix

> **lastIndexedAtUnix**: `number`

Defined in: [modules/bitview-client/index.js:654](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L654)

Unix timestamp of the last indexed block

***

### service

> **service**: `"brk"`

Defined in: [modules/bitview-client/index.js:643](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L643)

***

### startedAt

> **startedAt**: `string`

Defined in: [modules/bitview-client/index.js:646](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L646)

Server start time (ISO 8601)

***

### status

> **status**: `"healthy"`

Defined in: [modules/bitview-client/index.js:642](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L642)

***

### timestamp

> **timestamp**: `string`

Defined in: [modules/bitview-client/index.js:645](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L645)

Current server time (ISO 8601)

***

### tipHeight

> **tipHeight**: `number`

Defined in: [modules/bitview-client/index.js:650](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L650)

Height of the Bitcoin node's chain tip; `/health` reports `indexed_height` instead, without
asking the node

***

### uptimeSeconds

> **uptimeSeconds**: `number`

Defined in: [modules/bitview-client/index.js:647](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L647)

Uptime in seconds

***

### version

> **version**: `string`

Defined in: [modules/bitview-client/index.js:644](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L644)
