[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / CpfpCluster

# Interface: CpfpCluster

Defined in: [modules/bitview-client/index.js:365](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L365)

## Properties

### chunkIndex

> **chunkIndex**: `number`

Defined in: [modules/bitview-client/index.js:368](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L368)

Index into `chunks` of the chunk containing the seed tx.

***

### chunks

> **chunks**: [`CpfpClusterChunk`](CpfpClusterChunk.md)[]

Defined in: [modules/bitview-client/index.js:367](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L367)

Chunks ordered by descending feerate.

***

### txs

> **txs**: [`CpfpClusterTx`](CpfpClusterTx.md)[]

Defined in: [modules/bitview-client/index.js:366](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L366)

All txs in the cluster, in topological order (parents before children).
