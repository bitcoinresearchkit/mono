[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / ConfirmedCpfp

# Interface: ConfirmedCpfp

Defined in: [modules/bitview-client/index.js:325](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L325)

## Properties

### ancestors

> **ancestors**: [`CpfpEntry`](CpfpEntry.md)[]

Defined in: [modules/bitview-client/index.js:326](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L326)

***

### cluster

> **cluster**: [`CpfpCluster`](CpfpCluster.md)

Defined in: [modules/bitview-client/index.js:330](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L330)

The transaction's relatives in its block.

***

### descendants

> **descendants**: [`CpfpEntry`](CpfpEntry.md)[]

Defined in: [modules/bitview-client/index.js:327](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L327)

***

### effectiveFeePerVsize

> **effectiveFeePerVsize**: `number`

Defined in: [modules/bitview-client/index.js:328](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L328)

Feerate of the chunk holding the transaction when its block's relatives are linearized
  (sat/vB).
