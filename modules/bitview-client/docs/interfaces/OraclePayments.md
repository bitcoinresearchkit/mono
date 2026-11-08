[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / OraclePayments

# Interface: OraclePayments

Defined in: [modules/bitview-client/index.js:956](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L956)

## Properties

### blocks

> **blocks**: [`OracleBlockPayments`](OracleBlockPayments.md)[]

Defined in: [modules/bitview-client/index.js:959](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L959)

Each block's, in order.

***

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:957](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L957)

The last block's hash: it commits to every block before it.

***

### step

> **step**: `number`

Defined in: [modules/bitview-client/index.js:958](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L958)

A bucket's width in oracle bins: 0.1, a price range of 0.115%.
