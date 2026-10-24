[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / CpfpCluster

# Interface: CpfpCluster

Defined in: [modules/bitview-client/index.js:426](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L426)

## Properties

### chunkIndex

> **chunkIndex**: `number`

Defined in: [modules/bitview-client/index.js:429](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L429)

Index into `chunks` of the chunk containing the seed tx.

***

### chunks

> **chunks**: [`CpfpClusterChunk`](CpfpClusterChunk.md)[]

Defined in: [modules/bitview-client/index.js:428](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L428)

Chunks ordered by descending feerate.

***

### txs

> **txs**: [`CpfpClusterTx`](CpfpClusterTx.md)[]

Defined in: [modules/bitview-client/index.js:427](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L427)

All txs in the cluster, in topological order (parents before children).
