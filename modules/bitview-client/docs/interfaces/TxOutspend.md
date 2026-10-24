[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxOutspend

# Interface: TxOutspend

Defined in: [modules/bitview-client/index.js:1365](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1365)

## Properties

### spent

> **spent**: `boolean`

Defined in: [modules/bitview-client/index.js:1366](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1366)

***

### status?

> `optional` **status?**: [`TxStatus`](TxStatus.md) \| `null`

Defined in: [modules/bitview-client/index.js:1369](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1369)

Status of the spending transaction (only present if spent)

***

### txid?

> `optional` **txid?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1367](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1367)

Transaction ID of the spending transaction (only present if spent)

***

### vin?

> `optional` **vin?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1368](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1368)

Input index in the spending transaction (only present if spent)
