[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / CpfpCluster

# Interface: CpfpCluster

Defined in: [modules/bitview-client/index.js:412](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L412)

## Properties

### chunkIndex

> **chunkIndex**: `number`

Defined in: [modules/bitview-client/index.js:415](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L415)

Index into `chunks` of the chunk containing the seed tx.

***

### chunks

> **chunks**: [`CpfpClusterChunk`](CpfpClusterChunk.md)[]

Defined in: [modules/bitview-client/index.js:414](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L414)

Chunks ordered by descending feerate.

***

### txs

> **txs**: [`CpfpClusterTx`](CpfpClusterTx.md)[]

Defined in: [modules/bitview-client/index.js:413](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L413)

All txs in the cluster, in topological order (parents before children).
