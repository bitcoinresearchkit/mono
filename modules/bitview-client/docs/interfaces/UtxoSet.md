[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / UtxoSet

# Interface: UtxoSet

Defined in: [modules/bitview-client/index.js:1607](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1607)

## Properties

### count

> **count**: `number`

Defined in: [modules/bitview-client/index.js:1612](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1612)

Number of unspent outputs.

***

### date

> **date**: `string`

Defined in: [modules/bitview-client/index.js:1610](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1610)

UTC date of the represented block's monotonic timestamp (the running maximum of block
times).

***

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:1609](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1609)

***

### height

> **height**: `number`

Defined in: [modules/bitview-client/index.js:1608](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1608)

***

### origins

> **origins**: [`UtxoOrigins`](UtxoOrigins.md)

Defined in: [modules/bitview-client/index.js:1614](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1614)

Unspent outputs by creation height: entry `i` is what remains of block `i`'s outputs.

***

### supply

> **supply**: `number`

Defined in: [modules/bitview-client/index.js:1613](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1613)

BTC held by the unspent outputs.
