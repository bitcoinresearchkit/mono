[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / UtxoSet

# Interface: UtxoSet

Defined in: [modules/bitview-client/index.js:1456](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1456)

## Properties

### count

> **count**: `number`

Defined in: [modules/bitview-client/index.js:1461](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1461)

Number of unspent outputs.

***

### date

> **date**: `string`

Defined in: [modules/bitview-client/index.js:1459](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1459)

UTC date of the represented block's monotonic timestamp (the running maximum of block
times).

***

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:1458](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1458)

***

### height

> **height**: `number`

Defined in: [modules/bitview-client/index.js:1457](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1457)

***

### origins

> **origins**: [`UtxoOrigins`](UtxoOrigins.md)

Defined in: [modules/bitview-client/index.js:1463](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1463)

Unspent outputs by creation height: entry `i` is what remains of block `i`'s outputs.

***

### supply

> **supply**: `number`

Defined in: [modules/bitview-client/index.js:1462](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1462)

BTC held by the unspent outputs.
