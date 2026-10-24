[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxOutspend

# Interface: TxOutspend

Defined in: [modules/bitview-client/index.js:1449](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1449)

## Properties

### spent

> **spent**: `boolean`

Defined in: [modules/bitview-client/index.js:1450](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1450)

***

### status?

> `optional` **status?**: [`TxStatus`](TxStatus.md) \| `null`

Defined in: [modules/bitview-client/index.js:1453](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1453)

Status of the spending transaction (only present if spent)

***

### txid?

> `optional` **txid?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1451](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1451)

Transaction ID of the spending transaction (only present if spent)

***

### vin?

> `optional` **vin?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1452](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1452)

Input index in the spending transaction (only present if spent)
