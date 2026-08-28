[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / AddrStats

# Interface: AddrStats

Defined in: [Developer/brk/modules/bitview-client/index.js:77](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L77)

## Properties

### address

> **address**: `string`

Defined in: [Developer/brk/modules/bitview-client/index.js:78](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L78)

Bitcoin address string

***

### addrType

> **addrType**: [`OutputType`](../type-aliases/OutputType.md)

Defined in: [Developer/brk/modules/bitview-client/index.js:79](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L79)

BRK address type (p2pk33, p2pk65, p2pkh, p2sh, p2wpkh, p2wsh, p2tr, etc.)

***

### balance

> **balance**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:82](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L82)

Total current balance in satoshis, including pending (unconfirmed) mempool changes

***

### chainStats

> **chainStats**: [`AddrChainStats`](AddrChainStats.md)

Defined in: [Developer/brk/modules/bitview-client/index.js:80](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L80)

Statistics for confirmed transactions on the blockchain

***

### mempoolStats

> **mempoolStats**: [`AddrMempoolStats`](AddrMempoolStats.md)

Defined in: [Developer/brk/modules/bitview-client/index.js:81](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L81)

Statistics for unconfirmed transactions in the mempool
