[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / CpfpInfo

# Interface: CpfpInfo

Defined in: [modules/bitview-client/index.js:463](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L463)

## Properties

### adjustedVsize

> **adjustedVsize**: `number`

Defined in: [modules/bitview-client/index.js:474](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L474)

Policy-adjusted virtual size: `max(vsize, sigops * 5)`.

***

### ancestors

> **ancestors**: [`CpfpEntry`](CpfpEntry.md)[]

Defined in: [modules/bitview-client/index.js:464](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L464)

***

### bestDescendant?

> `optional` **bestDescendant?**: [`CpfpEntry`](CpfpEntry.md) \| `null`

Defined in: [modules/bitview-client/index.js:465](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L465)

Best (highest fee rate) descendant, if any.

***

### cluster?

> `optional` **cluster?**: [`CpfpCluster`](CpfpCluster.md) \| `null`

Defined in: [modules/bitview-client/index.js:475](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L475)

Cluster the seed belongs to; omitted when the seed has no ancestors and no descendants
(matches mempool.space).

***

### descendants

> **descendants**: [`CpfpEntry`](CpfpEntry.md)[]

Defined in: [modules/bitview-client/index.js:466](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L466)

***

### effectiveFeePerVsize

> **effectiveFeePerVsize**: `number`

Defined in: [modules/bitview-client/index.js:467](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L467)

Effective fee rate considering CPFP relationships (sat/vB).
This is the seed's chunk feerate after lift-merging, i.e. the
rate Core/mempool.space would surface for this tx.

***

### fee

> **fee**: `number`

Defined in: [modules/bitview-client/index.js:472](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L472)

Transaction fee (sats).

***

### sigops

> **sigops**: `number`

Defined in: [modules/bitview-client/index.js:470](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L470)

BIP-141 sigop cost for the seed tx (witness sigops count as 1,
legacy and P2SH-redeem sigops count as 4).

***

### vsize

> **vsize**: `number`

Defined in: [modules/bitview-client/index.js:473](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L473)

Virtual size of the seed tx (vbytes).
