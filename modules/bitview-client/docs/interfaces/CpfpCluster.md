[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / CpfpCluster

# Interface: CpfpCluster

Defined in: [modules/bitview-client/index.js:359](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L359)

## Properties

### chunkIndex

> **chunkIndex**: `number`

Defined in: [modules/bitview-client/index.js:362](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L362)

Index into `chunks` of the chunk containing the seed tx.

***

### chunks

> **chunks**: [`CpfpClusterChunk`](CpfpClusterChunk.md)[]

Defined in: [modules/bitview-client/index.js:361](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L361)

Chunks ordered by descending feerate.

***

### txs

> **txs**: [`CpfpClusterTx`](CpfpClusterTx.md)[]

Defined in: [modules/bitview-client/index.js:360](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L360)

All txs in the cluster, in topological order (parents before children).
