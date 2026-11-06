[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / AddressMempoolStats

# Interface: AddressMempoolStats

Defined in: [modules/bitview-client/index.js:35](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L35)

## Properties

### balanceDelta

> **balanceDelta**: `number`

Defined in: [modules/bitview-client/index.js:36](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L36)

Net pending (unconfirmed) balance change in satoshis; negative when pending spends exceed receipts

***

### fundedTxoCount

> **fundedTxoCount**: `number`

Defined in: [modules/bitview-client/index.js:37](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L37)

Number of unconfirmed transaction outputs funding this address

***

### fundedTxoSum

> **fundedTxoSum**: `number`

Defined in: [modules/bitview-client/index.js:38](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L38)

Total amount in satoshis being received in unconfirmed transactions

***

### spentTxoCount

> **spentTxoCount**: `number`

Defined in: [modules/bitview-client/index.js:39](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L39)

Number of unconfirmed transaction inputs spending from this address

***

### spentTxoSum

> **spentTxoSum**: `number`

Defined in: [modules/bitview-client/index.js:40](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L40)

Total amount in satoshis being spent in unconfirmed transactions

***

### txCount

> **txCount**: `number`

Defined in: [modules/bitview-client/index.js:41](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L41)

Number of unconfirmed transactions involving this address
