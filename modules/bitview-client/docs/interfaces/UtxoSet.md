[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / UtxoSet

# Interface: UtxoSet

Defined in: [modules/bitview-client/index.js:1647](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1647)

## Properties

### count

> **count**: `number`

Defined in: [modules/bitview-client/index.js:1652](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1652)

Number of unspent outputs.

***

### date

> **date**: `string`

Defined in: [modules/bitview-client/index.js:1650](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1650)

UTC date of the represented block's monotonic timestamp (the running maximum of block
times).

***

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:1649](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1649)

***

### height

> **height**: `number`

Defined in: [modules/bitview-client/index.js:1648](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1648)

***

### origins

> **origins**: [`UtxoOrigins`](UtxoOrigins.md)

Defined in: [modules/bitview-client/index.js:1654](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1654)

Unspent outputs by creation height: entry `i` is what remains of block `i`'s outputs.

***

### supply

> **supply**: `number`

Defined in: [modules/bitview-client/index.js:1653](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1653)

BTC held by the unspent outputs.
