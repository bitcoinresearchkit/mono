[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / UtxoSet

# Interface: UtxoSet

Defined in: [modules/bitview-client/index.js:1566](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1566)

## Properties

### count

> **count**: `number`

Defined in: [modules/bitview-client/index.js:1571](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1571)

Number of unspent outputs.

***

### date

> **date**: `string`

Defined in: [modules/bitview-client/index.js:1569](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1569)

UTC date of the represented block's monotonic timestamp (the running maximum of block
  times).

***

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:1568](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1568)

***

### height

> **height**: `number`

Defined in: [modules/bitview-client/index.js:1567](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1567)

***

### origins

> **origins**: [`UtxoOrigins`](UtxoOrigins.md)

Defined in: [modules/bitview-client/index.js:1573](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1573)

Unspent outputs by creation height: entry `i` is what remains of block `i`'s outputs.

***

### supply

> **supply**: `number`

Defined in: [modules/bitview-client/index.js:1572](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1572)

BTC held by the unspent outputs.
