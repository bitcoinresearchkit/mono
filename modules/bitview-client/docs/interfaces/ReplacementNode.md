[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / ReplacementNode

# Interface: ReplacementNode

Defined in: [modules/bitview-client/index.js:1189](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1189)

## Properties

### fullRbf

> **fullRbf**: `boolean`

Defined in: [modules/bitview-client/index.js:1193](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1193)

Any predecessor in this subtree was non-signaling.

***

### interval?

> `optional` **interval?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1194](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1194)

Seconds between this node's `time` and the successor that
replaced it. Omitted on the root of an RBF response.

***

### mined?

> `optional` **mined?**: `boolean` \| `null`

Defined in: [modules/bitview-client/index.js:1196](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1196)

`Some(true)` iff this node's tx is currently confirmed. Absent
on serialization otherwise.

***

### replaces

> **replaces**: `ReplacementNode`[]

Defined in: [modules/bitview-client/index.js:1198](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1198)

***

### time

> **time**: `number`

Defined in: [modules/bitview-client/index.js:1191](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1191)

First-seen timestamp, duplicated here to match mempool.space's
on-the-wire shape.

***

### tx

> **tx**: [`RbfTx`](RbfTx.md)

Defined in: [modules/bitview-client/index.js:1190](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1190)
