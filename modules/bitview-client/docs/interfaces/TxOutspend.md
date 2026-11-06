[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxOutspend

# Interface: TxOutspend

Defined in: [modules/bitview-client/index.js:1356](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1356)

## Properties

### spent

> **spent**: `boolean`

Defined in: [modules/bitview-client/index.js:1357](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1357)

***

### status?

> `optional` **status?**: [`TxStatus`](TxStatus.md) \| `null`

Defined in: [modules/bitview-client/index.js:1360](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1360)

Status of the spending transaction (only present if spent)

***

### txid?

> `optional` **txid?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1358](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1358)

Transaction ID of the spending transaction (only present if spent)

***

### vin?

> `optional` **vin?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1359](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1359)

Input index in the spending transaction (only present if spent)
