[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / UtxoSet

# Interface: UtxoSet

Defined in: [modules/bitview-client/index.js:1551](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1551)

## Properties

### count

> **count**: `number`

Defined in: [modules/bitview-client/index.js:1556](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1556)

Number of unspent outputs.

***

### date

> **date**: `string`

Defined in: [modules/bitview-client/index.js:1554](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1554)

UTC date of the represented block's monotonic timestamp (the running maximum of block
times).

***

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:1553](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1553)

***

### height

> **height**: `number`

Defined in: [modules/bitview-client/index.js:1552](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1552)

***

### origins

> **origins**: [`UtxoOrigins`](UtxoOrigins.md)

Defined in: [modules/bitview-client/index.js:1558](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1558)

Unspent outputs by creation height: entry `i` is what remains of block `i`'s outputs.

***

### supply

> **supply**: `number`

Defined in: [modules/bitview-client/index.js:1557](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1557)

BTC held by the unspent outputs.
