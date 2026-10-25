[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxOutspend

# Interface: TxOutspend

Defined in: [modules/bitview-client/index.js:1489](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1489)

## Properties

### spent

> **spent**: `boolean`

Defined in: [modules/bitview-client/index.js:1490](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1490)

***

### status?

> `optional` **status?**: [`TxStatus`](TxStatus.md) \| `null`

Defined in: [modules/bitview-client/index.js:1493](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1493)

Status of the spending transaction (only present if spent)

***

### txid?

> `optional` **txid?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1491](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1491)

Transaction ID of the spending transaction (only present if spent)

***

### vin?

> `optional` **vin?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1492](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1492)

Input index in the spending transaction (only present if spent)
