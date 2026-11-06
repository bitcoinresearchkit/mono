[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxOutspend

# Interface: TxOutspend

Defined in: [modules/bitview-client/index.js:1501](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1501)

## Properties

### spent

> **spent**: `boolean`

Defined in: [modules/bitview-client/index.js:1502](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1502)

***

### status?

> `optional` **status?**: [`TxStatus`](TxStatus.md) \| `null`

Defined in: [modules/bitview-client/index.js:1505](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1505)

Status of the spending transaction (only present if spent)

***

### txid?

> `optional` **txid?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:1503](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1503)

Transaction ID of the spending transaction (only present if spent)

***

### vin?

> `optional` **vin?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1504](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1504)

Input index in the spending transaction (only present if spent)
