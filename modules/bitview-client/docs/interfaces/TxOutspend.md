[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxOutspend

# Interface: TxOutspend

Defined in: [modules/bitview-client/index.js:1387](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1387)

## Properties

### spent

> **spent**: `boolean`

Defined in: [modules/bitview-client/index.js:1388](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1388)

***

### status?

> `optional` **status?**: [`TxStatus`](TxStatus.md) \| `null`

Defined in: [modules/bitview-client/index.js:1391](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1391)

Status of the spending transaction (only present if spent)

***

### txid?

> `optional` **txid?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1389](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1389)

Transaction ID of the spending transaction (only present if spent)

***

### vin?

> `optional` **vin?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1390](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1390)

Input index in the spending transaction (only present if spent)
