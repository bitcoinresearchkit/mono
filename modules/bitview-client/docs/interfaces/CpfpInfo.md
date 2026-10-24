[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / CpfpInfo

# Interface: CpfpInfo

Defined in: [modules/bitview-client/index.js:478](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L478)

## Properties

### adjustedVsize

> **adjustedVsize**: `number`

Defined in: [modules/bitview-client/index.js:489](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L489)

Policy-adjusted virtual size: `max(vsize, sigops * 5)`.

***

### ancestors

> **ancestors**: [`CpfpEntry`](CpfpEntry.md)[]

Defined in: [modules/bitview-client/index.js:479](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L479)

***

### bestDescendant?

> `optional` **bestDescendant?**: [`CpfpEntry`](CpfpEntry.md) \| `null`

Defined in: [modules/bitview-client/index.js:480](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L480)

Best (highest fee rate) descendant, if any.

***

### cluster?

> `optional` **cluster?**: [`CpfpCluster`](CpfpCluster.md) \| `null`

Defined in: [modules/bitview-client/index.js:490](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L490)

Cluster the seed belongs to; omitted when the seed has no ancestors and no descendants
(matches mempool.space).

***

### descendants

> **descendants**: [`CpfpEntry`](CpfpEntry.md)[]

Defined in: [modules/bitview-client/index.js:481](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L481)

***

### effectiveFeePerVsize

> **effectiveFeePerVsize**: `number`

Defined in: [modules/bitview-client/index.js:482](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L482)

Effective fee rate considering CPFP relationships (sat/vB).
This is the seed's chunk feerate after lift-merging, i.e. the
rate Core/mempool.space would surface for this tx.

***

### fee

> **fee**: `number`

Defined in: [modules/bitview-client/index.js:487](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L487)

Transaction fee (sats).

***

### sigops

> **sigops**: `number`

Defined in: [modules/bitview-client/index.js:485](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L485)

BIP-141 sigop cost for the seed tx (witness sigops count as 1,
legacy and P2SH-redeem sigops count as 4).

***

### vsize

> **vsize**: `number`

Defined in: [modules/bitview-client/index.js:488](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L488)

Virtual size of the seed tx (vbytes).
