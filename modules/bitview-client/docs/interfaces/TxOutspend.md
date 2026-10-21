[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxOutspend

# Interface: TxOutspend

Defined in: [modules/bitview-client/index.js:1288](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1288)

## Properties

### spent

> **spent**: `boolean`

Defined in: [modules/bitview-client/index.js:1289](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1289)

***

### status?

> `optional` **status?**: [`TxStatus`](TxStatus.md) \| `null`

Defined in: [modules/bitview-client/index.js:1292](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1292)

Status of the spending transaction (only present if spent)

***

### txid?

> `optional` **txid?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1290](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1290)

Transaction ID of the spending transaction (only present if spent)

***

### vin?

> `optional` **vin?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1291](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1291)

Input index in the spending transaction (only present if spent)
