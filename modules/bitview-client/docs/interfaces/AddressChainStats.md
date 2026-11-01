[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / AddressChainStats

# Interface: AddressChainStats

Defined in: modules/bitview-client/index.js:19

## Properties

### balance

> **balance**: `number`

Defined in: modules/bitview-client/index.js:20

Current confirmed balance in satoshis

***

### fundedTxoCount

> **fundedTxoCount**: `number`

Defined in: modules/bitview-client/index.js:21

Total number of transaction outputs that funded this address

***

### fundedTxoSum

> **fundedTxoSum**: `number`

Defined in: modules/bitview-client/index.js:22

Total amount in satoshis received by this address across all funded outputs

***

### realizedPrice

> **realizedPrice**: `number`

Defined in: modules/bitview-client/index.js:28

Realized price (average cost basis) in USD

***

### spentTxoCount

> **spentTxoCount**: `number`

Defined in: modules/bitview-client/index.js:23

Total number of transaction outputs spent from this address

***

### spentTxoSum

> **spentTxoSum**: `number`

Defined in: modules/bitview-client/index.js:24

Total amount in satoshis spent from this address

***

### txCount

> **txCount**: `number`

Defined in: modules/bitview-client/index.js:25

Total number of confirmed transactions involving this address

***

### typeIndex?

> `optional` **typeIndex?**: `number` \| `null`

Defined in: modules/bitview-client/index.js:26

Index of this address within its type on the blockchain, null for an address without
confirmed activity
