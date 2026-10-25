[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / ConfirmedCpfp

# Interface: ConfirmedCpfp

Defined in: [modules/bitview-client/index.js:372](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L372)

## Properties

### ancestors

> **ancestors**: [`CpfpEntry`](CpfpEntry.md)[]

Defined in: [modules/bitview-client/index.js:373](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L373)

***

### cluster

> **cluster**: [`CpfpCluster`](CpfpCluster.md)

Defined in: [modules/bitview-client/index.js:377](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L377)

The transaction's relatives in its block.

***

### descendants

> **descendants**: [`CpfpEntry`](CpfpEntry.md)[]

Defined in: [modules/bitview-client/index.js:374](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L374)

***

### effectiveFeePerVsize

> **effectiveFeePerVsize**: `number`

Defined in: [modules/bitview-client/index.js:375](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L375)

Feerate of the chunk holding the transaction when its block's relatives are linearized
(sat/vB).
