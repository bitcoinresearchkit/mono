[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxOutspend

# Interface: TxOutspend

Defined in: [modules/bitview-client/index.js:1467](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1467)

## Properties

### spent

> **spent**: `boolean`

Defined in: [modules/bitview-client/index.js:1468](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1468)

***

### status?

> `optional` **status?**: [`TxStatus`](TxStatus.md) \| `null`

Defined in: [modules/bitview-client/index.js:1471](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1471)

Status of the spending transaction (only present if spent)

***

### txid?

> `optional` **txid?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1469](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1469)

Transaction ID of the spending transaction (only present if spent)

***

### vin?

> `optional` **vin?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1470](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1470)

Input index in the spending transaction (only present if spent)
