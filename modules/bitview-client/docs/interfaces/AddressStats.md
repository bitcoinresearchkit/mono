[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / AddressStats

# Interface: AddressStats

Defined in: [modules/bitview-client/index.js:73](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L73)

## Properties

### address

> **address**: `string`

Defined in: [modules/bitview-client/index.js:74](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L74)

Bitcoin address, or the hex public key of a P2PK output

***

### addressType

> **addressType**: [`OutputType`](../type-aliases/OutputType.md)

Defined in: [modules/bitview-client/index.js:75](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L75)

Output script type

***

### balance

> **balance**: `number`

Defined in: [modules/bitview-client/index.js:78](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L78)

Total current balance in satoshis, including pending (unconfirmed) mempool changes

***

### chainStats

> **chainStats**: [`AddressChainStats`](AddressChainStats.md)

Defined in: [modules/bitview-client/index.js:76](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L76)

Statistics for confirmed transactions on the blockchain

***

### mempoolStats

> **mempoolStats**: [`AddressMempoolStats`](AddressMempoolStats.md)

Defined in: [modules/bitview-client/index.js:77](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L77)

Statistics for unconfirmed transactions in the mempool
