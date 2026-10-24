[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / ConfirmedCpfp

# Interface: ConfirmedCpfp

Defined in: [modules/bitview-client/index.js:392](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L392)

## Properties

### ancestors

> **ancestors**: [`CpfpEntry`](CpfpEntry.md)[]

Defined in: [modules/bitview-client/index.js:393](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L393)

***

### cluster

> **cluster**: [`CpfpCluster`](CpfpCluster.md)

Defined in: [modules/bitview-client/index.js:397](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L397)

The transaction's relatives in its block.

***

### descendants

> **descendants**: [`CpfpEntry`](CpfpEntry.md)[]

Defined in: [modules/bitview-client/index.js:394](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L394)

***

### effectiveFeePerVsize

> **effectiveFeePerVsize**: `number`

Defined in: [modules/bitview-client/index.js:395](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L395)

Feerate of the chunk holding the transaction when its block's relatives are linearized
(sat/vB).
