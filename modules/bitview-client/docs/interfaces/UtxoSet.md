[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / UtxoSet

# Interface: UtxoSet

Defined in: [modules/bitview-client/index.js:1545](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1545)

## Properties

### count

> **count**: `number`

Defined in: [modules/bitview-client/index.js:1550](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1550)

Number of unspent outputs.

***

### date

> **date**: `string`

Defined in: [modules/bitview-client/index.js:1548](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1548)

UTC date of the represented block's monotonic timestamp (the running maximum of block
times).

***

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:1547](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1547)

***

### height

> **height**: `number`

Defined in: [modules/bitview-client/index.js:1546](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1546)

***

### origins

> **origins**: [`UtxoOrigins`](UtxoOrigins.md)

Defined in: [modules/bitview-client/index.js:1552](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1552)

Unspent outputs by creation height: entry `i` is what remains of block `i`'s outputs.

***

### supply

> **supply**: `number`

Defined in: [modules/bitview-client/index.js:1551](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1551)

BTC held by the unspent outputs.
