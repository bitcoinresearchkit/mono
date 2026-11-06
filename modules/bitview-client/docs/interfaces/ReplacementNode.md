[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / ReplacementNode

# Interface: ReplacementNode

Defined in: [modules/bitview-client/index.js:1179](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1179)

## Properties

### fullRbf

> **fullRbf**: `boolean`

Defined in: [modules/bitview-client/index.js:1183](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1183)

Any predecessor in this subtree was non-signaling.

***

### interval?

> `optional` **interval?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1184](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1184)

Seconds between this node's `time` and the successor that
  replaced it. Omitted on the root of an RBF response.

***

### mined?

> `optional` **mined?**: `boolean` \| `null`

Defined in: [modules/bitview-client/index.js:1186](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1186)

`Some(true)` iff this node's tx is currently confirmed. Absent
  on serialization otherwise.

***

### replaces

> **replaces**: `ReplacementNode`[]

Defined in: [modules/bitview-client/index.js:1188](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1188)

***

### time

> **time**: `number`

Defined in: [modules/bitview-client/index.js:1181](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1181)

First-seen timestamp, duplicated here to match mempool.space's
  on-the-wire shape.

***

### tx

> **tx**: [`RbfTx`](RbfTx.md)

Defined in: [modules/bitview-client/index.js:1180](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1180)
