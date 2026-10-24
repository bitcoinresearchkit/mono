[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / CpfpCluster

# Interface: CpfpCluster

Defined in: [modules/bitview-client/index.js:416](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L416)

## Properties

### chunkIndex

> **chunkIndex**: `number`

Defined in: [modules/bitview-client/index.js:419](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L419)

Index into `chunks` of the chunk containing the seed tx.

***

### chunks

> **chunks**: [`CpfpClusterChunk`](CpfpClusterChunk.md)[]

Defined in: [modules/bitview-client/index.js:418](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L418)

Chunks ordered by descending feerate.

***

### txs

> **txs**: [`CpfpClusterTx`](CpfpClusterTx.md)[]

Defined in: [modules/bitview-client/index.js:417](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L417)

All txs in the cluster, in topological order (parents before children).
