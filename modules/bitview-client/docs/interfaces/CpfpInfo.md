[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / CpfpInfo

# Interface: CpfpInfo

Defined in: [modules/bitview-client/index.js:396](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L396)

## Properties

### adjustedVsize

> **adjustedVsize**: `number`

Defined in: [modules/bitview-client/index.js:407](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L407)

Policy-adjusted virtual size: `max(vsize, sigops * 5)`.

***

### ancestors

> **ancestors**: [`CpfpEntry`](CpfpEntry.md)[]

Defined in: [modules/bitview-client/index.js:397](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L397)

***

### bestDescendant?

> `optional` **bestDescendant?**: [`CpfpEntry`](CpfpEntry.md) \| `null`

Defined in: [modules/bitview-client/index.js:398](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L398)

Best (highest fee rate) descendant, if any.

***

### cluster?

> `optional` **cluster?**: [`CpfpCluster`](CpfpCluster.md) \| `null`

Defined in: [modules/bitview-client/index.js:408](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L408)

Cluster the seed belongs to; omitted when the seed has no ancestors and no descendants
  (matches mempool.space).

***

### descendants

> **descendants**: [`CpfpEntry`](CpfpEntry.md)[]

Defined in: [modules/bitview-client/index.js:399](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L399)

***

### effectiveFeePerVsize

> **effectiveFeePerVsize**: `number`

Defined in: [modules/bitview-client/index.js:400](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L400)

Effective fee rate considering CPFP relationships (sat/vB).
  This is the seed's chunk feerate after lift-merging, i.e. the
  rate Core/mempool.space would surface for this tx.

***

### fee

> **fee**: `number`

Defined in: [modules/bitview-client/index.js:405](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L405)

Transaction fee (sats).

***

### sigops

> **sigops**: `number`

Defined in: [modules/bitview-client/index.js:403](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L403)

BIP-141 sigop cost for the seed tx (witness sigops count as 1,
  legacy and P2SH-redeem sigops count as 4).

***

### vsize

> **vsize**: `number`

Defined in: [modules/bitview-client/index.js:406](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L406)

Virtual size of the seed tx (vbytes).
