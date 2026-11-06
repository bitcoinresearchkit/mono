[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / AddressStats

# Interface: AddressStats

Defined in: [modules/bitview-client/index.js:51](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L51)

## Properties

### address

> **address**: `string`

Defined in: [modules/bitview-client/index.js:52](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L52)

Bitcoin address, or the hex public key of a P2PK output

***

### addressType

> **addressType**: [`OutputType`](../type-aliases/OutputType.md)

Defined in: [modules/bitview-client/index.js:53](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L53)

Output script type

***

### balance

> **balance**: `number`

Defined in: [modules/bitview-client/index.js:56](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L56)

Total current balance in satoshis, including pending (unconfirmed) mempool changes

***

### chainStats

> **chainStats**: [`AddressChainStats`](AddressChainStats.md)

Defined in: [modules/bitview-client/index.js:54](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L54)

Statistics for confirmed transactions on the blockchain

***

### mempoolStats

> **mempoolStats**: [`AddressMempoolStats`](AddressMempoolStats.md)

Defined in: [modules/bitview-client/index.js:55](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L55)

Statistics for unconfirmed transactions in the mempool
