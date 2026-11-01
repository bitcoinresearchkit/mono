[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / CpfpCluster

# Interface: CpfpCluster

Defined in: modules/bitview-client/index.js:406

## Properties

### chunkIndex

> **chunkIndex**: `number`

Defined in: modules/bitview-client/index.js:409

Index into `chunks` of the chunk containing the seed tx.

***

### chunks

> **chunks**: [`CpfpClusterChunk`](CpfpClusterChunk.md)[]

Defined in: modules/bitview-client/index.js:408

Chunks ordered by descending feerate.

***

### txs

> **txs**: [`CpfpClusterTx`](CpfpClusterTx.md)[]

Defined in: modules/bitview-client/index.js:407

All txs in the cluster, in topological order (parents before children).
