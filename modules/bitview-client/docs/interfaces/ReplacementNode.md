[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / ReplacementNode

# Interface: ReplacementNode

Defined in: modules/bitview-client/index.js:1277

## Properties

### fullRbf

> **fullRbf**: `boolean`

Defined in: modules/bitview-client/index.js:1281

Any predecessor in this subtree was non-signaling.

***

### interval?

> `optional` **interval?**: `number` \| `null`

Defined in: modules/bitview-client/index.js:1282

Seconds between this node's `time` and the successor that
replaced it. Omitted on the root of an RBF response.

***

### mined?

> `optional` **mined?**: `boolean` \| `null`

Defined in: modules/bitview-client/index.js:1284

`Some(true)` iff this node's tx is currently confirmed. Absent
on serialization otherwise.

***

### replaces

> **replaces**: `ReplacementNode`[]

Defined in: modules/bitview-client/index.js:1286

***

### time

> **time**: `number`

Defined in: modules/bitview-client/index.js:1279

First-seen timestamp, duplicated here to match mempool.space's
on-the-wire shape.

***

### tx

> **tx**: [`RbfTx`](RbfTx.md)

Defined in: modules/bitview-client/index.js:1278
