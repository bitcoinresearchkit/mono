[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / AddressChainStats

# Interface: AddressChainStats

Defined in: [modules/bitview-client/index.js:14](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L14)

## Properties

### balance

> **balance**: `number`

Defined in: [modules/bitview-client/index.js:15](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L15)

Current confirmed balance in satoshis

***

### fundedTxoCount

> **fundedTxoCount**: `number`

Defined in: [modules/bitview-client/index.js:16](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L16)

Total number of transaction outputs that funded this address

***

### fundedTxoSum

> **fundedTxoSum**: `number`

Defined in: [modules/bitview-client/index.js:17](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L17)

Total amount in satoshis received by this address across all funded outputs

***

### realizedPrice

> **realizedPrice**: `number`

Defined in: [modules/bitview-client/index.js:23](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L23)

Realized price (average cost basis) in USD

***

### spentTxoCount

> **spentTxoCount**: `number`

Defined in: [modules/bitview-client/index.js:18](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L18)

Total number of transaction outputs spent from this address

***

### spentTxoSum

> **spentTxoSum**: `number`

Defined in: [modules/bitview-client/index.js:19](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L19)

Total amount in satoshis spent from this address

***

### txCount

> **txCount**: `number`

Defined in: [modules/bitview-client/index.js:20](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L20)

Total number of confirmed transactions involving this address

***

### typeIndex?

> `optional` **typeIndex?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:21](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L21)

Index of this address within its type on the blockchain, null for an address without
  confirmed activity
