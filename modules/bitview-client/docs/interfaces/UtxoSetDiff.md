[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / UtxoSetDiff

# Interface: UtxoSetDiff

Defined in: [modules/bitview-client/index.js:1587](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1587)

## Properties

### created

> **created**: [`UtxoChanges`](UtxoChanges.md)

Defined in: [modules/bitview-client/index.js:1593](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1593)

Outputs created, one row per block, including those spent again within the range (which
`spent` lists too).

***

### date

> **date**: `string`

Defined in: [modules/bitview-client/index.js:1592](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1592)

UTC date of block `last`'s monotonic timestamp.

***

### first

> **first**: `number`

Defined in: [modules/bitview-client/index.js:1588](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1588)

First block of the diff; it applies to the set after block `first - 1` (the empty set for
block 0).

***

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:1591](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1591)

Hash of block `last`.

***

### last

> **last**: `number`

Defined in: [modules/bitview-client/index.js:1590](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1590)

Last block of the diff, described by `hash` and `date`.

***

### spent

> **spent**: [`UtxoChanges`](UtxoChanges.md)

Defined in: [modules/bitview-client/index.js:1595](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1595)

Outputs removed, one row per creation height in ascending order: spent outputs, plus the
coinbase outputs of blocks 91812 and 91722, which duplicates at 91842 and 91880 overwrote.
