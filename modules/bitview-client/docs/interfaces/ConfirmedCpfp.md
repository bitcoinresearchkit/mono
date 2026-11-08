[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / ConfirmedCpfp

# Interface: ConfirmedCpfp

Defined in: [modules/bitview-client/index.js:383](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L383)

## Properties

### ancestors

> **ancestors**: [`CpfpEntry`](CpfpEntry.md)[]

Defined in: [modules/bitview-client/index.js:384](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L384)

***

### cluster

> **cluster**: [`CpfpCluster`](CpfpCluster.md)

Defined in: [modules/bitview-client/index.js:388](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L388)

The transaction's relatives in its block.

***

### descendants

> **descendants**: [`CpfpEntry`](CpfpEntry.md)[]

Defined in: [modules/bitview-client/index.js:385](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L385)

***

### effectiveFeePerVsize

> **effectiveFeePerVsize**: `number`

Defined in: [modules/bitview-client/index.js:386](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L386)

Feerate of the chunk holding the transaction when its block's relatives are linearized
  (sat/vB).
