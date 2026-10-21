[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxOutspend

# Interface: TxOutspend

Defined in: [modules/bitview-client/index.js:1298](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1298)

## Properties

### spent

> **spent**: `boolean`

Defined in: [modules/bitview-client/index.js:1299](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1299)

***

### status?

> `optional` **status?**: [`TxStatus`](TxStatus.md) \| `null`

Defined in: [modules/bitview-client/index.js:1302](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1302)

Status of the spending transaction (only present if spent)

***

### txid?

> `optional` **txid?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1300](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1300)

Transaction ID of the spending transaction (only present if spent)

***

### vin?

> `optional` **vin?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1301](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1301)

Input index in the spending transaction (only present if spent)
