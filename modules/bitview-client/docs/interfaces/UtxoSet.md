[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / UtxoSet

# Interface: UtxoSet

Defined in: [modules/bitview-client/index.js:1523](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1523)

## Properties

### count

> **count**: `number`

Defined in: [modules/bitview-client/index.js:1528](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1528)

Number of unspent outputs.

***

### date

> **date**: `string`

Defined in: [modules/bitview-client/index.js:1526](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1526)

UTC date of the represented block's monotonic timestamp (the running maximum of block
times).

***

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:1525](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1525)

***

### height

> **height**: `number`

Defined in: [modules/bitview-client/index.js:1524](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1524)

***

### origins

> **origins**: [`UtxoOrigins`](UtxoOrigins.md)

Defined in: [modules/bitview-client/index.js:1530](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1530)

Unspent outputs by creation height: entry `i` is what remains of block `i`'s outputs.

***

### supply

> **supply**: `number`

Defined in: [modules/bitview-client/index.js:1529](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1529)

BTC held by the unspent outputs.
