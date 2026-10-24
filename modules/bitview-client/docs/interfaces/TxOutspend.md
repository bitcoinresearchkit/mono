[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxOutspend

# Interface: TxOutspend

Defined in: [modules/bitview-client/index.js:1393](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1393)

## Properties

### spent

> **spent**: `boolean`

Defined in: [modules/bitview-client/index.js:1394](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1394)

***

### status?

> `optional` **status?**: [`TxStatus`](TxStatus.md) \| `null`

Defined in: [modules/bitview-client/index.js:1397](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1397)

Status of the spending transaction (only present if spent)

***

### txid?

> `optional` **txid?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1395](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1395)

Transaction ID of the spending transaction (only present if spent)

***

### vin?

> `optional` **vin?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1396](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1396)

Input index in the spending transaction (only present if spent)
