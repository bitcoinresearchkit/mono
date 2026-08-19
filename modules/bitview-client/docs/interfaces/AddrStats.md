[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / AddrStats

# Interface: AddrStats

Defined in: [Developer/brk/modules/bitview-client/index.js:68](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L68)

## Properties

### address

> **address**: `string`

Defined in: [Developer/brk/modules/bitview-client/index.js:69](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L69)

Bitcoin address string

***

### addrType

> **addrType**: [`OutputType`](../type-aliases/OutputType.md)

Defined in: [Developer/brk/modules/bitview-client/index.js:70](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L70)

BRK address type (p2pk33, p2pk65, p2pkh, p2sh, p2wpkh, p2wsh, p2tr, etc.)

***

### balance

> **balance**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:73](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L73)

Total current balance in satoshis, including pending (unconfirmed) mempool changes

***

### chainStats

> **chainStats**: [`AddrChainStats`](AddrChainStats.md)

Defined in: [Developer/brk/modules/bitview-client/index.js:71](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L71)

Statistics for confirmed transactions on the blockchain

***

### mempoolStats

> **mempoolStats**: [`AddrMempoolStats`](AddrMempoolStats.md)

Defined in: [Developer/brk/modules/bitview-client/index.js:72](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L72)

Statistics for unconfirmed transactions in the mempool
