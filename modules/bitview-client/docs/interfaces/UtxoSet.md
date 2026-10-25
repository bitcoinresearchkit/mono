[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / UtxoSet

# Interface: UtxoSet

Defined in: [modules/bitview-client/index.js:1625](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1625)

## Properties

### count

> **count**: `number`

Defined in: [modules/bitview-client/index.js:1630](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1630)

Number of unspent outputs.

***

### date

> **date**: `string`

Defined in: [modules/bitview-client/index.js:1628](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1628)

UTC date of the represented block's monotonic timestamp (the running maximum of block
times).

***

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:1627](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1627)

***

### height

> **height**: `number`

Defined in: [modules/bitview-client/index.js:1626](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1626)

***

### origins

> **origins**: [`UtxoOrigins`](UtxoOrigins.md)

Defined in: [modules/bitview-client/index.js:1632](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1632)

Unspent outputs by creation height: entry `i` is what remains of block `i`'s outputs.

***

### supply

> **supply**: `number`

Defined in: [modules/bitview-client/index.js:1631](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1631)

BTC held by the unspent outputs.
