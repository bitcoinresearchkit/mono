[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxOutspend

# Interface: TxOutspend

Defined in: [modules/bitview-client/index.js:1417](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1417)

## Properties

### spent

> **spent**: `boolean`

Defined in: [modules/bitview-client/index.js:1418](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1418)

***

### status?

> `optional` **status?**: [`TxStatus`](TxStatus.md) \| `null`

Defined in: [modules/bitview-client/index.js:1421](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1421)

Status of the spending transaction (only present if spent)

***

### txid?

> `optional` **txid?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1419](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1419)

Transaction ID of the spending transaction (only present if spent)

***

### vin?

> `optional` **vin?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1420](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1420)

Input index in the spending transaction (only present if spent)
