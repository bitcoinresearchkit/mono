[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxOutspend

# Interface: TxOutspend

Defined in: modules/bitview-client/index.js:1493

## Properties

### spent

> **spent**: `boolean`

Defined in: modules/bitview-client/index.js:1494

***

### status?

> `optional` **status?**: [`TxStatus`](TxStatus.md) \| `null`

Defined in: modules/bitview-client/index.js:1497

Status of the spending transaction (only present if spent)

***

### txid?

> `optional` **txid?**: `string` \| `null`

Defined in: modules/bitview-client/index.js:1495

Transaction ID of the spending transaction (only present if spent)

***

### vin?

> `optional` **vin?**: `number` \| `null`

Defined in: modules/bitview-client/index.js:1496

Input index in the spending transaction (only present if spent)
