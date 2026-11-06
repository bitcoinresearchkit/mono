[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / OraclePayments

# Interface: OraclePayments

Defined in: [modules/bitview-client/index.js:982](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L982)

## Properties

### blocks

> **blocks**: [`OracleBlockPayments`](OracleBlockPayments.md)[]

Defined in: [modules/bitview-client/index.js:985](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L985)

Each block's, in order.

***

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:983](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L983)

The last block's hash: it commits to every block before it.

***

### step

> **step**: `number`

Defined in: [modules/bitview-client/index.js:984](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L984)

A bucket's width in oracle bins: 0.1, a price range of 0.115%.
