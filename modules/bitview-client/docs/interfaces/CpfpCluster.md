[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / CpfpCluster

# Interface: CpfpCluster

Defined in: [modules/bitview-client/index.js:401](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L401)

## Properties

### chunkIndex

> **chunkIndex**: `number`

Defined in: [modules/bitview-client/index.js:404](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L404)

Index into `chunks` of the chunk containing the seed tx.

***

### chunks

> **chunks**: [`CpfpClusterChunk`](CpfpClusterChunk.md)[]

Defined in: [modules/bitview-client/index.js:403](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L403)

Chunks ordered by descending feerate.

***

### txs

> **txs**: [`CpfpClusterTx`](CpfpClusterTx.md)[]

Defined in: [modules/bitview-client/index.js:402](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L402)

All txs in the cluster, in topological order (parents before children).
