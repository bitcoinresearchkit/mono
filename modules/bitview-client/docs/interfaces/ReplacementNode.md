[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / ReplacementNode

# Interface: ReplacementNode

Defined in: [modules/bitview-client/index.js:1207](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1207)

## Properties

### fullRbf

> **fullRbf**: `boolean`

Defined in: [modules/bitview-client/index.js:1211](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1211)

Any predecessor in this subtree was non-signaling.

***

### interval?

> `optional` **interval?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1212](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1212)

Seconds between this node's `time` and the successor that
replaced it. Omitted on the root of an RBF response.

***

### mined?

> `optional` **mined?**: `boolean` \| `null`

Defined in: [modules/bitview-client/index.js:1214](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1214)

`Some(true)` iff this node's tx is currently confirmed. Absent
on serialization otherwise.

***

### replaces

> **replaces**: `ReplacementNode`[]

Defined in: [modules/bitview-client/index.js:1216](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1216)

***

### time

> **time**: `number`

Defined in: [modules/bitview-client/index.js:1209](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1209)

First-seen timestamp, duplicated here to match mempool.space's
on-the-wire shape.

***

### tx

> **tx**: [`RbfTx`](RbfTx.md)

Defined in: [modules/bitview-client/index.js:1208](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1208)
