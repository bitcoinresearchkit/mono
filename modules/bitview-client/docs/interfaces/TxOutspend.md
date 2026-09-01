[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxOutspend

# Interface: TxOutspend

Defined in: [Developer/mono/modules/bitview-client/index.js:1432](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1432)

## Properties

### spent

> **spent**: `boolean`

Defined in: [Developer/mono/modules/bitview-client/index.js:1433](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1433)

Whether the output has been spent

***

### status?

> `optional` **status?**: [`TxStatus`](TxStatus.md) \| `null`

Defined in: [Developer/mono/modules/bitview-client/index.js:1436](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1436)

Status of the spending transaction (only present if spent)

***

### txid?

> `optional` **txid?**: `string` \| `null`

Defined in: [Developer/mono/modules/bitview-client/index.js:1434](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1434)

Transaction ID of the spending transaction (only present if spent)

***

### vin?

> `optional` **vin?**: `number` \| `null`

Defined in: [Developer/mono/modules/bitview-client/index.js:1435](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1435)

Input index in the spending transaction (only present if spent)
