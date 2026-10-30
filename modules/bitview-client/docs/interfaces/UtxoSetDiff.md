[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / UtxoSetDiff

# Interface: UtxoSetDiff

Defined in: modules/bitview-client/index.js:1663

## Properties

### created

> **created**: [`UtxoChanges`](UtxoChanges.md)

Defined in: modules/bitview-client/index.js:1669

Outputs created, one row per block, including those spent again within the range (which
`spent` lists too).

***

### date

> **date**: `string`

Defined in: modules/bitview-client/index.js:1668

UTC date of block `last`'s monotonic timestamp.

***

### first

> **first**: `number`

Defined in: modules/bitview-client/index.js:1664

First block of the diff; it applies to the set after block `first - 1` (the empty set for
block 0).

***

### hash

> **hash**: `string`

Defined in: modules/bitview-client/index.js:1667

Hash of block `last`.

***

### last

> **last**: `number`

Defined in: modules/bitview-client/index.js:1666

Last block of the diff, described by `hash` and `date`.

***

### spent

> **spent**: [`UtxoChanges`](UtxoChanges.md)

Defined in: modules/bitview-client/index.js:1671

Outputs removed, one row per creation height in ascending order: spent outputs, plus the
coinbase outputs of blocks 91812 and 91722, which duplicates at 91842 and 91880 overwrote.
