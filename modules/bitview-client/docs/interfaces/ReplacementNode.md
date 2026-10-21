[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / ReplacementNode

# Interface: ReplacementNode

Defined in: [modules/bitview-client/index.js:1094](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1094)

## Properties

### fullRbf

> **fullRbf**: `boolean`

Defined in: [modules/bitview-client/index.js:1098](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1098)

Any predecessor in this subtree was non-signaling.

***

### interval?

> `optional` **interval?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1099](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1099)

Seconds between this node's `time` and the successor that
replaced it. Omitted on the root of an RBF response.

***

### mined?

> `optional` **mined?**: `boolean` \| `null`

Defined in: [modules/bitview-client/index.js:1101](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1101)

`Some(true)` iff this node's tx is currently confirmed. Absent
on serialization otherwise.

***

### replaces

> **replaces**: `ReplacementNode`[]

Defined in: [modules/bitview-client/index.js:1103](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1103)

***

### time

> **time**: `number`

Defined in: [modules/bitview-client/index.js:1096](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1096)

First-seen timestamp, duplicated here to match mempool.space's
on-the-wire shape.

***

### tx

> **tx**: [`RbfTx`](RbfTx.md)

Defined in: [modules/bitview-client/index.js:1095](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1095)
