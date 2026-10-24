[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / ConfirmedCpfp

# Interface: ConfirmedCpfp

Defined in: [modules/bitview-client/index.js:407](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L407)

## Properties

### ancestors

> **ancestors**: [`CpfpEntry`](CpfpEntry.md)[]

Defined in: [modules/bitview-client/index.js:408](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L408)

***

### cluster

> **cluster**: [`CpfpCluster`](CpfpCluster.md)

Defined in: [modules/bitview-client/index.js:412](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L412)

The transaction's relatives in its block.

***

### descendants

> **descendants**: [`CpfpEntry`](CpfpEntry.md)[]

Defined in: [modules/bitview-client/index.js:409](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L409)

***

### effectiveFeePerVsize

> **effectiveFeePerVsize**: `number`

Defined in: [modules/bitview-client/index.js:410](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L410)

Feerate of the chunk holding the transaction when its block's relatives are linearized
(sat/vB).
