[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / AddrMempoolStats

# Interface: AddrMempoolStats

Defined in: [Developer/brk/modules/bitview-client/index.js:51](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L51)

## Properties

### balanceDelta

> **balanceDelta**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:52](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L52)

Net pending (unconfirmed) balance change in satoshis; negative when pending spends exceed receipts

***

### fundedTxoCount

> **fundedTxoCount**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:53](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L53)

Number of unconfirmed transaction outputs funding this address

***

### fundedTxoSum

> **fundedTxoSum**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:54](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L54)

Total amount in satoshis being received in unconfirmed transactions

***

### spentTxoCount

> **spentTxoCount**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:55](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L55)

Number of unconfirmed transaction inputs spending from this address

***

### spentTxoSum

> **spentTxoSum**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:56](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L56)

Total amount in satoshis being spent in unconfirmed transactions

***

### txCount

> **txCount**: `number`

Defined in: [Developer/brk/modules/bitview-client/index.js:57](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L57)

Number of unconfirmed transactions involving this address
