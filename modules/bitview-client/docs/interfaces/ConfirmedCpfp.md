[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / ConfirmedCpfp

# Interface: ConfirmedCpfp

Defined in: [modules/bitview-client/index.js:382](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L382)

## Properties

### ancestors

> **ancestors**: [`CpfpEntry`](CpfpEntry.md)[]

Defined in: [modules/bitview-client/index.js:383](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L383)

***

### cluster

> **cluster**: [`CpfpCluster`](CpfpCluster.md)

Defined in: [modules/bitview-client/index.js:387](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L387)

The transaction's relatives in its block.

***

### descendants

> **descendants**: [`CpfpEntry`](CpfpEntry.md)[]

Defined in: [modules/bitview-client/index.js:384](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L384)

***

### effectiveFeePerVsize

> **effectiveFeePerVsize**: `number`

Defined in: [modules/bitview-client/index.js:385](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L385)

Feerate of the chunk holding the transaction when its block's relatives are linearized
(sat/vB).
