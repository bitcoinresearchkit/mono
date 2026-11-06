[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / Health

# Interface: Health

Defined in: [modules/bitview-client/index.js:592](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L592)

## Properties

### blocksBehind

> **blocksBehind**: `number`

Defined in: [modules/bitview-client/index.js:603](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L603)

Number of blocks behind the tip

***

### computedHeight

> **computedHeight**: `number`

Defined in: [modules/bitview-client/index.js:600](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L600)

Height of the last computed block (series)

***

### indexedHeight

> **indexedHeight**: `number`

Defined in: [modules/bitview-client/index.js:599](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L599)

Height of the last indexed block

***

### lastIndexedAt

> **lastIndexedAt**: `string`

Defined in: [modules/bitview-client/index.js:604](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L604)

Human-readable timestamp of the last indexed block (ISO 8601)

***

### lastIndexedAtUnix

> **lastIndexedAtUnix**: `number`

Defined in: [modules/bitview-client/index.js:605](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L605)

Unix timestamp of the last indexed block

***

### service

> **service**: `"brk"`

Defined in: [modules/bitview-client/index.js:594](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L594)

***

### startedAt

> **startedAt**: `string`

Defined in: [modules/bitview-client/index.js:597](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L597)

Server start time (ISO 8601)

***

### status

> **status**: `"healthy"`

Defined in: [modules/bitview-client/index.js:593](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L593)

***

### timestamp

> **timestamp**: `string`

Defined in: [modules/bitview-client/index.js:596](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L596)

Current server time (ISO 8601)

***

### tipHeight

> **tipHeight**: `number`

Defined in: [modules/bitview-client/index.js:601](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L601)

Height of the Bitcoin node's chain tip; `/health` reports `indexed_height` instead, without
  asking the node

***

### uptimeSeconds

> **uptimeSeconds**: `number`

Defined in: [modules/bitview-client/index.js:598](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L598)

Uptime in seconds

***

### version

> **version**: `string`

Defined in: [modules/bitview-client/index.js:595](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L595)
