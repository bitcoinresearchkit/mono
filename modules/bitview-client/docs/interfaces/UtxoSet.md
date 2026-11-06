[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / UtxoSet

# Interface: UtxoSet

Defined in: [modules/bitview-client/index.js:1659](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1659)

## Properties

### count

> **count**: `number`

Defined in: [modules/bitview-client/index.js:1664](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1664)

Number of unspent outputs.

***

### date

> **date**: `string`

Defined in: [modules/bitview-client/index.js:1662](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1662)

UTC date of the represented block's monotonic timestamp (the running maximum of block
  times).

***

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:1661](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1661)

***

### height

> **height**: `number`

Defined in: [modules/bitview-client/index.js:1660](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1660)

***

### origins

> **origins**: [`UtxoOrigins`](UtxoOrigins.md)

Defined in: [modules/bitview-client/index.js:1666](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1666)

Unspent outputs by creation height: entry `i` is what remains of block `i`'s outputs.

***

### supply

> **supply**: `number`

Defined in: [modules/bitview-client/index.js:1665](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1665)

BTC held by the unspent outputs.
