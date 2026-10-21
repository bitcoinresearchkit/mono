[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / AddressMempoolStats

# Interface: AddressMempoolStats

Defined in: [modules/bitview-client/index.js:46](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L46)

## Properties

### balanceDelta

> **balanceDelta**: `number`

Defined in: [modules/bitview-client/index.js:47](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L47)

Net pending (unconfirmed) balance change in satoshis; negative when pending spends exceed receipts

***

### fundedTxoCount

> **fundedTxoCount**: `number`

Defined in: [modules/bitview-client/index.js:48](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L48)

Number of unconfirmed transaction outputs funding this address

***

### fundedTxoSum

> **fundedTxoSum**: `number`

Defined in: [modules/bitview-client/index.js:49](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L49)

Total amount in satoshis being received in unconfirmed transactions

***

### spentTxoCount

> **spentTxoCount**: `number`

Defined in: [modules/bitview-client/index.js:50](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L50)

Number of unconfirmed transaction inputs spending from this address

***

### spentTxoSum

> **spentTxoSum**: `number`

Defined in: [modules/bitview-client/index.js:51](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L51)

Total amount in satoshis being spent in unconfirmed transactions

***

### txCount

> **txCount**: `number`

Defined in: [modules/bitview-client/index.js:52](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L52)

Number of unconfirmed transactions involving this address
