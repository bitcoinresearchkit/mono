[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / AddressMempoolStats

# Interface: AddressMempoolStats

Defined in: [modules/bitview-client/index.js:52](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L52)

## Properties

### balanceDelta

> **balanceDelta**: `number`

Defined in: [modules/bitview-client/index.js:53](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L53)

Net pending (unconfirmed) balance change in satoshis; negative when pending spends exceed receipts

***

### fundedTxoCount

> **fundedTxoCount**: `number`

Defined in: [modules/bitview-client/index.js:54](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L54)

Number of unconfirmed transaction outputs funding this address

***

### fundedTxoSum

> **fundedTxoSum**: `number`

Defined in: [modules/bitview-client/index.js:55](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L55)

Total amount in satoshis being received in unconfirmed transactions

***

### spentTxoCount

> **spentTxoCount**: `number`

Defined in: [modules/bitview-client/index.js:56](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L56)

Number of unconfirmed transaction inputs spending from this address

***

### spentTxoSum

> **spentTxoSum**: `number`

Defined in: [modules/bitview-client/index.js:57](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L57)

Total amount in satoshis being spent in unconfirmed transactions

***

### txCount

> **txCount**: `number`

Defined in: [modules/bitview-client/index.js:58](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L58)

Number of unconfirmed transactions involving this address
