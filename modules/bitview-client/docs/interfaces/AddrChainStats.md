[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / AddrChainStats

# Interface: AddrChainStats

Defined in: [Developer/mono/modules/bitview-client/index.js:23](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L23)

## Properties

### balance

> **balance**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:24](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L24)

Current confirmed balance in satoshis

***

### fundedTxoCount

> **fundedTxoCount**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:25](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L25)

Total number of transaction outputs that funded this address

***

### fundedTxoSum

> **fundedTxoSum**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:26](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L26)

Total amount in satoshis received by this address across all funded outputs

***

### realizedPrice

> **realizedPrice**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:31](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L31)

Realized price (average cost basis) in USD

***

### spentTxoCount

> **spentTxoCount**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:27](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L27)

Total number of transaction outputs spent from this address

***

### spentTxoSum

> **spentTxoSum**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:28](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L28)

Total amount in satoshis spent from this address

***

### txCount

> **txCount**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:29](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L29)

Total number of confirmed transactions involving this address

***

### typeIndex

> **typeIndex**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:30](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L30)

Index of this address within its type on the blockchain
