[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / UtxoSet

# Interface: UtxoSet

Defined in: modules/bitview-client/index.js:1651

## Properties

### count

> **count**: `number`

Defined in: modules/bitview-client/index.js:1656

Number of unspent outputs.

***

### date

> **date**: `string`

Defined in: modules/bitview-client/index.js:1654

UTC date of the represented block's monotonic timestamp (the running maximum of block
times).

***

### hash

> **hash**: `string`

Defined in: modules/bitview-client/index.js:1653

***

### height

> **height**: `number`

Defined in: modules/bitview-client/index.js:1652

***

### origins

> **origins**: [`UtxoOrigins`](UtxoOrigins.md)

Defined in: modules/bitview-client/index.js:1658

Unspent outputs by creation height: entry `i` is what remains of block `i`'s outputs.

***

### supply

> **supply**: `number`

Defined in: modules/bitview-client/index.js:1657

BTC held by the unspent outputs.
