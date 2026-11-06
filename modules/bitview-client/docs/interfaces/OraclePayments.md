[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / OraclePayments

# Interface: OraclePayments

Defined in: [modules/bitview-client/index.js:890](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L890)

## Properties

### blocks

> **blocks**: [`OracleBlockPayments`](OracleBlockPayments.md)[]

Defined in: [modules/bitview-client/index.js:893](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L893)

Each block's, in order.

***

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:891](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L891)

The last block's hash: it commits to every block before it.

***

### step

> **step**: `number`

Defined in: [modules/bitview-client/index.js:892](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L892)

A bucket's width in oracle bins: 0.1, a price range of 0.115%.
