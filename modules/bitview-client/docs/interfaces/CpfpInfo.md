[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / CpfpInfo

# Interface: CpfpInfo

Defined in: [modules/bitview-client/index.js:443](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L443)

## Properties

### adjustedVsize

> **adjustedVsize**: `number`

Defined in: [modules/bitview-client/index.js:454](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L454)

Policy-adjusted virtual size: `max(vsize, sigops * 5)`.

***

### ancestors

> **ancestors**: [`CpfpEntry`](CpfpEntry.md)[]

Defined in: [modules/bitview-client/index.js:444](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L444)

***

### bestDescendant?

> `optional` **bestDescendant?**: [`CpfpEntry`](CpfpEntry.md) \| `null`

Defined in: [modules/bitview-client/index.js:445](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L445)

Best (highest fee rate) descendant, if any.

***

### cluster?

> `optional` **cluster?**: [`CpfpCluster`](CpfpCluster.md) \| `null`

Defined in: [modules/bitview-client/index.js:455](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L455)

Cluster the seed belongs to; omitted when the seed has no ancestors and no descendants
(matches mempool.space).

***

### descendants

> **descendants**: [`CpfpEntry`](CpfpEntry.md)[]

Defined in: [modules/bitview-client/index.js:446](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L446)

***

### effectiveFeePerVsize

> **effectiveFeePerVsize**: `number`

Defined in: [modules/bitview-client/index.js:447](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L447)

Effective fee rate considering CPFP relationships (sat/vB).
This is the seed's chunk feerate after lift-merging, i.e. the
rate Core/mempool.space would surface for this tx.

***

### fee

> **fee**: `number`

Defined in: [modules/bitview-client/index.js:452](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L452)

Transaction fee (sats).

***

### sigops

> **sigops**: `number`

Defined in: [modules/bitview-client/index.js:450](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L450)

BIP-141 sigop cost for the seed tx (witness sigops count as 1,
legacy and P2SH-redeem sigops count as 4).

***

### vsize

> **vsize**: `number`

Defined in: [modules/bitview-client/index.js:453](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L453)

Virtual size of the seed tx (vbytes).
