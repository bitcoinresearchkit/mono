[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / AddressStats

# Interface: AddressStats

Defined in: modules/bitview-client/index.js:78

## Properties

### address

> **address**: `string`

Defined in: modules/bitview-client/index.js:79

Bitcoin address, or the hex public key of a P2PK output

***

### addressType

> **addressType**: [`OutputType`](../type-aliases/OutputType.md)

Defined in: modules/bitview-client/index.js:80

Output script type

***

### balance

> **balance**: `number`

Defined in: modules/bitview-client/index.js:83

Total current balance in satoshis, including pending (unconfirmed) mempool changes

***

### chainStats

> **chainStats**: [`AddressChainStats`](AddressChainStats.md)

Defined in: modules/bitview-client/index.js:81

Statistics for confirmed transactions on the blockchain

***

### mempoolStats

> **mempoolStats**: [`AddressMempoolStats`](AddressMempoolStats.md)

Defined in: modules/bitview-client/index.js:82

Statistics for unconfirmed transactions in the mempool
