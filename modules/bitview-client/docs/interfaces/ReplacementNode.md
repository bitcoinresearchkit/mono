[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / ReplacementNode

# Interface: ReplacementNode

Defined in: [modules/bitview-client/index.js:1246](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1246)

## Properties

### fullRbf

> **fullRbf**: `boolean`

Defined in: [modules/bitview-client/index.js:1250](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1250)

Any predecessor in this subtree was non-signaling.

***

### interval?

> `optional` **interval?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1251](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1251)

Seconds between this node's `time` and the successor that
  replaced it. Omitted on the root of an RBF response.

***

### mined?

> `optional` **mined?**: `boolean` \| `null`

Defined in: [modules/bitview-client/index.js:1253](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1253)

On a tree's root: `true` when a transaction anywhere in the tree is confirmed
  (mempool.space's tree `mined`; the confirmed one is marked in its `tx`); absent otherwise.

***

### replaces

> **replaces**: `ReplacementNode`[]

Defined in: [modules/bitview-client/index.js:1258](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1258)

***

### time

> **time**: `number`

Defined in: [modules/bitview-client/index.js:1248](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1248)

First-seen timestamp, duplicated here to match mempool.space's
  on-the-wire shape.

***

### truncated?

> `optional` **truncated?**: `boolean` \| `null`

Defined in: [modules/bitview-client/index.js:1255](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1255)

On a tree's root: `true` when the tree was cut to its bounds (the newest 63 versions, at
  most 4,096 transactions; its `fullRbf` flags then describe what it holds); absent
  otherwise. mempool.space serves trees whole.

***

### tx

> **tx**: [`RbfTx`](RbfTx.md)

Defined in: [modules/bitview-client/index.js:1247](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1247)
