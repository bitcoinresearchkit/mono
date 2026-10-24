[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / ReplacementNode

# Interface: ReplacementNode

Defined in: [modules/bitview-client/index.js:1239](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1239)

## Properties

### fullRbf

> **fullRbf**: `boolean`

Defined in: [modules/bitview-client/index.js:1243](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1243)

Any predecessor in this subtree was non-signaling.

***

### interval?

> `optional` **interval?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1244](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1244)

Seconds between this node's `time` and the successor that
replaced it. Omitted on the root of an RBF response.

***

### mined?

> `optional` **mined?**: `boolean` \| `null`

Defined in: [modules/bitview-client/index.js:1246](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1246)

`Some(true)` iff this node's tx is currently confirmed. Absent
on serialization otherwise.

***

### replaces

> **replaces**: `ReplacementNode`[]

Defined in: [modules/bitview-client/index.js:1248](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1248)

***

### time

> **time**: `number`

Defined in: [modules/bitview-client/index.js:1241](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1241)

First-seen timestamp, duplicated here to match mempool.space's
on-the-wire shape.

***

### tx

> **tx**: [`RbfTx`](RbfTx.md)

Defined in: [modules/bitview-client/index.js:1240](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1240)
