[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / AddressStats

# Interface: AddressStats

Defined in: [modules/bitview-client/index.js:61](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L61)

## Properties

### address

> **address**: `string`

Defined in: [modules/bitview-client/index.js:62](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L62)

Bitcoin address, or the hex public key of a P2PK output

***

### addressType

> **addressType**: [`OutputType`](../type-aliases/OutputType.md)

Defined in: [modules/bitview-client/index.js:63](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L63)

Output script type

***

### balance

> **balance**: `number`

Defined in: [modules/bitview-client/index.js:66](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L66)

Total current balance in satoshis, including pending (unconfirmed) mempool changes

***

### chainStats

> **chainStats**: [`AddressChainStats`](AddressChainStats.md)

Defined in: [modules/bitview-client/index.js:64](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L64)

Statistics for confirmed transactions on the blockchain

***

### mempoolStats

> **mempoolStats**: [`AddressMempoolStats`](AddressMempoolStats.md)

Defined in: [modules/bitview-client/index.js:65](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L65)

Statistics for unconfirmed transactions in the mempool
