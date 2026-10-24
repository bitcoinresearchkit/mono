[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / ConfirmedCpfp

# Interface: ConfirmedCpfp

Defined in: [modules/bitview-client/index.js:367](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L367)

## Properties

### ancestors

> **ancestors**: [`CpfpEntry`](CpfpEntry.md)[]

Defined in: [modules/bitview-client/index.js:368](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L368)

***

### cluster

> **cluster**: [`CpfpCluster`](CpfpCluster.md)

Defined in: [modules/bitview-client/index.js:372](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L372)

The transaction's relatives in its block.

***

### descendants

> **descendants**: [`CpfpEntry`](CpfpEntry.md)[]

Defined in: [modules/bitview-client/index.js:369](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L369)

***

### effectiveFeePerVsize

> **effectiveFeePerVsize**: `number`

Defined in: [modules/bitview-client/index.js:370](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L370)

Feerate of the chunk holding the transaction when its block's relatives are linearized
(sat/vB).
