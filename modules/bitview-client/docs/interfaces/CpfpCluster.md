[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / CpfpCluster

# Interface: CpfpCluster

Defined in: [modules/bitview-client/index.js:441](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L441)

## Properties

### chunkIndex

> **chunkIndex**: `number`

Defined in: [modules/bitview-client/index.js:444](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L444)

Index into `chunks` of the chunk containing the seed tx.

***

### chunks

> **chunks**: [`CpfpClusterChunk`](CpfpClusterChunk.md)[]

Defined in: [modules/bitview-client/index.js:443](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L443)

Chunks ordered by descending feerate.

***

### txs

> **txs**: [`CpfpClusterTx`](CpfpClusterTx.md)[]

Defined in: [modules/bitview-client/index.js:442](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L442)

All txs in the cluster, in topological order (parents before children).
