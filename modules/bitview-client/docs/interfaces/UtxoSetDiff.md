[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / UtxoSetDiff

# Interface: UtxoSetDiff

Defined in: [modules/bitview-client/index.js:1578](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1578)

## Properties

### created

> **created**: [`UtxoChanges`](UtxoChanges.md)

Defined in: [modules/bitview-client/index.js:1584](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1584)

Outputs created, one row per block, including those spent again within the range (which
  `spent` lists too).

***

### date

> **date**: `string`

Defined in: [modules/bitview-client/index.js:1583](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1583)

UTC date of block `last`'s monotonic timestamp.

***

### first

> **first**: `number`

Defined in: [modules/bitview-client/index.js:1579](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1579)

First block of the diff; it applies to the set after block `first - 1` (the empty set for
  block 0).

***

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:1582](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1582)

Hash of block `last`.

***

### last

> **last**: `number`

Defined in: [modules/bitview-client/index.js:1581](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1581)

Last block of the diff, described by `hash` and `date`.

***

### spent

> **spent**: [`UtxoChanges`](UtxoChanges.md)

Defined in: [modules/bitview-client/index.js:1586](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1586)

Outputs removed, one row per creation height in ascending order: spent outputs, plus the
  coinbase outputs of blocks 91812 and 91722, which duplicates at 91842 and 91880 overwrote.
