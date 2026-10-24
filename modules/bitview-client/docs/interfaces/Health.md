[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / Health

# Interface: Health

Defined in: [modules/bitview-client/index.js:681](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L681)

## Properties

### blocksBehind

> **blocksBehind**: `number`

Defined in: [modules/bitview-client/index.js:692](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L692)

Number of blocks behind the tip

***

### computedHeight

> **computedHeight**: `number`

Defined in: [modules/bitview-client/index.js:689](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L689)

Height of the last computed block (series)

***

### indexedHeight

> **indexedHeight**: `number`

Defined in: [modules/bitview-client/index.js:688](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L688)

Height of the last indexed block

***

### lastIndexedAt

> **lastIndexedAt**: `string`

Defined in: [modules/bitview-client/index.js:693](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L693)

Human-readable timestamp of the last indexed block (ISO 8601)

***

### lastIndexedAtUnix

> **lastIndexedAtUnix**: `number`

Defined in: [modules/bitview-client/index.js:694](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L694)

Unix timestamp of the last indexed block

***

### service

> **service**: `"brk"`

Defined in: [modules/bitview-client/index.js:683](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L683)

***

### startedAt

> **startedAt**: `string`

Defined in: [modules/bitview-client/index.js:686](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L686)

Server start time (ISO 8601)

***

### status

> **status**: `"healthy"`

Defined in: [modules/bitview-client/index.js:682](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L682)

***

### timestamp

> **timestamp**: `string`

Defined in: [modules/bitview-client/index.js:685](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L685)

Current server time (ISO 8601)

***

### tipHeight

> **tipHeight**: `number`

Defined in: [modules/bitview-client/index.js:690](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L690)

Height of the Bitcoin node's chain tip; `/health` reports `indexed_height` instead, without
asking the node

***

### uptimeSeconds

> **uptimeSeconds**: `number`

Defined in: [modules/bitview-client/index.js:687](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L687)

Uptime in seconds

***

### version

> **version**: `string`

Defined in: [modules/bitview-client/index.js:684](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L684)
