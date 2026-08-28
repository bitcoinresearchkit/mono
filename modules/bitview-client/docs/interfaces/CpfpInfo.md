[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / CpfpInfo

# Interface: CpfpInfo

Defined in: [Developer/brk/modules/bitview-client/index.js:470](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L470)

## Properties

### adjustedVsize

> **adjustedVsize**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:481](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L481)

Policy-adjusted virtual size: `max(vsize, sigops * 5)`.

***

### ancestors

> **ancestors**: [`CpfpEntry`](CpfpEntry.md)[]

Defined in: [Developer/brk/modules/bitview-client/index.js:471](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L471)

Ancestor transactions in the CPFP chain.

***

### bestDescendant?

> `optional` **bestDescendant?**: [`CpfpEntry`](CpfpEntry.md) \| `null`

Defined in: [Developer/brk/modules/bitview-client/index.js:472](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L472)

Best (highest fee rate) descendant, if any.

***

### cluster?

> `optional` **cluster?**: [`CpfpCluster`](CpfpCluster.md) \| `null`

Defined in: [Developer/brk/modules/bitview-client/index.js:482](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L482)

Cluster the seed belongs to: full tx list, SFL-linearized chunks,
and the seed's chunk index. Omitted when the seed has no
ancestors and no descendants (matches mempool.space).

***

### descendants

> **descendants**: [`CpfpEntry`](CpfpEntry.md)[]

Defined in: [Developer/brk/modules/bitview-client/index.js:473](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L473)

Descendant transactions in the CPFP chain.

***

### effectiveFeePerVsize

> **effectiveFeePerVsize**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:474](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L474)

Effective fee rate considering CPFP relationships (sat/vB).
This is the seed's chunk feerate after lift-merging, i.e. the
rate Core/mempool.space would surface for this tx.

***

### fee

> **fee**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:479](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L479)

Transaction fee (sats).

***

### sigops

> **sigops**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:477](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L477)

BIP-141 sigop cost for the seed tx (witness sigops count as 1,
legacy and P2SH-redeem sigops count as 4).

***

### vsize

> **vsize**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:480](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L480)

Virtual size of the seed tx (vbytes).
