[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / UtxoSet

# Interface: UtxoSet

Defined in: [modules/bitview-client/index.js:1557](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1557)

## Properties

### count

> **count**: `number`

Defined in: [modules/bitview-client/index.js:1562](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1562)

Number of unspent outputs.

***

### date

> **date**: `string`

Defined in: [modules/bitview-client/index.js:1560](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1560)

UTC date of the represented block's monotonic timestamp (the running maximum of block
times).

***

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:1559](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1559)

***

### height

> **height**: `number`

Defined in: [modules/bitview-client/index.js:1558](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1558)

***

### origins

> **origins**: [`UtxoOrigins`](UtxoOrigins.md)

Defined in: [modules/bitview-client/index.js:1564](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1564)

Unspent outputs by creation height: entry `i` is what remains of block `i`'s outputs.

***

### supply

> **supply**: `number`

Defined in: [modules/bitview-client/index.js:1563](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1563)

BTC held by the unspent outputs.
