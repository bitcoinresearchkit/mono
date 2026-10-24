[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxOutspend

# Interface: TxOutspend

Defined in: [modules/bitview-client/index.js:1399](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1399)

## Properties

### spent

> **spent**: `boolean`

Defined in: [modules/bitview-client/index.js:1400](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1400)

***

### status?

> `optional` **status?**: [`TxStatus`](TxStatus.md) \| `null`

Defined in: [modules/bitview-client/index.js:1403](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1403)

Status of the spending transaction (only present if spent)

***

### txid?

> `optional` **txid?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1401](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1401)

Transaction ID of the spending transaction (only present if spent)

***

### vin?

> `optional` **vin?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1402](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1402)

Input index in the spending transaction (only present if spent)
