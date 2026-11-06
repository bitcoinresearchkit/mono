[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / ConfirmedCpfp

# Interface: ConfirmedCpfp

Defined in: [modules/bitview-client/index.js:378](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L378)

## Properties

### ancestors

> **ancestors**: [`CpfpEntry`](CpfpEntry.md)[]

Defined in: [modules/bitview-client/index.js:379](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L379)

***

### cluster

> **cluster**: [`CpfpCluster`](CpfpCluster.md)

Defined in: [modules/bitview-client/index.js:383](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L383)

The transaction's relatives in its block.

***

### descendants

> **descendants**: [`CpfpEntry`](CpfpEntry.md)[]

Defined in: [modules/bitview-client/index.js:380](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L380)

***

### effectiveFeePerVsize

> **effectiveFeePerVsize**: `number`

Defined in: [modules/bitview-client/index.js:381](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L381)

Feerate of the chunk holding the transaction when its block's relatives are linearized
  (sat/vB).
