[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / UtxoSetDiff

# Interface: UtxoSetDiff

Defined in: [modules/bitview-client/index.js:1671](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1671)

## Properties

### created

> **created**: [`UtxoChanges`](UtxoChanges.md)

Defined in: [modules/bitview-client/index.js:1677](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1677)

Outputs created, one row per block, including those spent again within the range (which
  `spent` lists too).

***

### date

> **date**: `string`

Defined in: [modules/bitview-client/index.js:1676](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1676)

UTC date of block `last`'s monotonic timestamp.

***

### first

> **first**: `number`

Defined in: [modules/bitview-client/index.js:1672](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1672)

First block of the diff; it applies to the set after block `first - 1` (the empty set for
  block 0).

***

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:1675](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1675)

Hash of block `last`.

***

### last

> **last**: `number`

Defined in: [modules/bitview-client/index.js:1674](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1674)

Last block of the diff, described by `hash` and `date`.

***

### spent

> **spent**: [`UtxoChanges`](UtxoChanges.md)

Defined in: [modules/bitview-client/index.js:1679](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1679)

Outputs removed, one row per creation height in ascending order: spent outputs, plus the
  coinbase outputs of blocks 91812 and 91722, which duplicates at 91842 and 91880 overwrote.
