[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / AddressChainStats

# Interface: AddressChainStats

Defined in: [modules/bitview-client/index.js:19](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L19)

## Properties

### balance

> **balance**: `number`

Defined in: [modules/bitview-client/index.js:20](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L20)

Current confirmed balance in satoshis

***

### fundedTxoCount

> **fundedTxoCount**: `number`

Defined in: [modules/bitview-client/index.js:21](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L21)

Total number of transaction outputs that funded this address

***

### fundedTxoSum

> **fundedTxoSum**: `number`

Defined in: [modules/bitview-client/index.js:22](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L22)

Total amount in satoshis received by this address across all funded outputs

***

### realizedPrice

> **realizedPrice**: `number`

Defined in: [modules/bitview-client/index.js:28](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L28)

Realized price (average cost basis) in USD

***

### spentTxoCount

> **spentTxoCount**: `number`

Defined in: [modules/bitview-client/index.js:23](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L23)

Total number of transaction outputs spent from this address

***

### spentTxoSum

> **spentTxoSum**: `number`

Defined in: [modules/bitview-client/index.js:24](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L24)

Total amount in satoshis spent from this address

***

### txCount

> **txCount**: `number`

Defined in: [modules/bitview-client/index.js:25](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L25)

Total number of confirmed transactions involving this address

***

### typeIndex?

> `optional` **typeIndex?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:26](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L26)

Index of this address within its type on the blockchain, null for an address without
confirmed activity
