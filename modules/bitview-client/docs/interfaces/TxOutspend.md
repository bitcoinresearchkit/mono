[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxOutspend

# Interface: TxOutspend

Defined in: [modules/bitview-client/index.js:1439](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1439)

## Properties

### spent

> **spent**: `boolean`

Defined in: [modules/bitview-client/index.js:1440](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1440)

***

### status?

> `optional` **status?**: [`TxStatus`](TxStatus.md) \| `null`

Defined in: [modules/bitview-client/index.js:1443](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1443)

Status of the spending transaction (only present if spent)

***

### txid?

> `optional` **txid?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1441](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1441)

Transaction ID of the spending transaction (only present if spent)

***

### vin?

> `optional` **vin?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1442](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1442)

Input index in the spending transaction (only present if spent)
