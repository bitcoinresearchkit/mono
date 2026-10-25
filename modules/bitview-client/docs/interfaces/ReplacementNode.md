[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / ReplacementNode

# Interface: ReplacementNode

Defined in: [modules/bitview-client/index.js:1275](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1275)

## Properties

### fullRbf

> **fullRbf**: `boolean`

Defined in: [modules/bitview-client/index.js:1279](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1279)

Any predecessor in this subtree was non-signaling.

***

### interval?

> `optional` **interval?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1280](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1280)

Seconds between this node's `time` and the successor that
replaced it. Omitted on the root of an RBF response.

***

### mined?

> `optional` **mined?**: `boolean` \| `null`

Defined in: [modules/bitview-client/index.js:1282](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1282)

`Some(true)` iff this node's tx is currently confirmed. Absent
on serialization otherwise.

***

### replaces

> **replaces**: `ReplacementNode`[]

Defined in: [modules/bitview-client/index.js:1284](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1284)

***

### time

> **time**: `number`

Defined in: [modules/bitview-client/index.js:1277](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1277)

First-seen timestamp, duplicated here to match mempool.space's
on-the-wire shape.

***

### tx

> **tx**: [`RbfTx`](RbfTx.md)

Defined in: [modules/bitview-client/index.js:1276](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1276)
