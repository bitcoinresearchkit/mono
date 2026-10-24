[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / UtxoSet

# Interface: UtxoSet

Defined in: [modules/bitview-client/index.js:1575](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1575)

## Properties

### count

> **count**: `number`

Defined in: [modules/bitview-client/index.js:1580](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1580)

Number of unspent outputs.

***

### date

> **date**: `string`

Defined in: [modules/bitview-client/index.js:1578](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1578)

UTC date of the represented block's monotonic timestamp (the running maximum of block
times).

***

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:1577](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1577)

***

### height

> **height**: `number`

Defined in: [modules/bitview-client/index.js:1576](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1576)

***

### origins

> **origins**: [`UtxoOrigins`](UtxoOrigins.md)

Defined in: [modules/bitview-client/index.js:1582](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1582)

Unspent outputs by creation height: entry `i` is what remains of block `i`'s outputs.

***

### supply

> **supply**: `number`

Defined in: [modules/bitview-client/index.js:1581](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1581)

BTC held by the unspent outputs.
