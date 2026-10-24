[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / ReplacementNode

# Interface: ReplacementNode

Defined in: [modules/bitview-client/index.js:1161](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1161)

## Properties

### fullRbf

> **fullRbf**: `boolean`

Defined in: [modules/bitview-client/index.js:1165](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1165)

Any predecessor in this subtree was non-signaling.

***

### interval?

> `optional` **interval?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1166](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1166)

Seconds between this node's `time` and the successor that
replaced it. Omitted on the root of an RBF response.

***

### mined?

> `optional` **mined?**: `boolean` \| `null`

Defined in: [modules/bitview-client/index.js:1168](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1168)

`Some(true)` iff this node's tx is currently confirmed. Absent
on serialization otherwise.

***

### replaces

> **replaces**: `ReplacementNode`[]

Defined in: [modules/bitview-client/index.js:1170](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1170)

***

### time

> **time**: `number`

Defined in: [modules/bitview-client/index.js:1163](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1163)

First-seen timestamp, duplicated here to match mempool.space's
on-the-wire shape.

***

### tx

> **tx**: [`RbfTx`](RbfTx.md)

Defined in: [modules/bitview-client/index.js:1162](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1162)
