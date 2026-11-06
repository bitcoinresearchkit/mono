[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / UtxoSet

# Interface: UtxoSet

Defined in: [modules/bitview-client/index.js:1483](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1483)

## Properties

### count

> **count**: `number`

Defined in: [modules/bitview-client/index.js:1488](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1488)

Number of unspent outputs.

***

### date

> **date**: `string`

Defined in: [modules/bitview-client/index.js:1486](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1486)

UTC date of the represented block's monotonic timestamp (the running maximum of block
  times).

***

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:1485](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1485)

***

### height

> **height**: `number`

Defined in: [modules/bitview-client/index.js:1484](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1484)

***

### origins

> **origins**: [`UtxoOrigins`](UtxoOrigins.md)

Defined in: [modules/bitview-client/index.js:1490](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1490)

Unspent outputs by creation height: entry `i` is what remains of block `i`'s outputs.

***

### supply

> **supply**: `number`

Defined in: [modules/bitview-client/index.js:1489](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1489)

BTC held by the unspent outputs.
