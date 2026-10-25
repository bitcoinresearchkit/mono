[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / OraclePayments

# Interface: OraclePayments

Defined in: [modules/bitview-client/index.js:972](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L972)

## Properties

### blocks

> **blocks**: [`OracleBlockPayments`](OracleBlockPayments.md)[]

Defined in: [modules/bitview-client/index.js:975](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L975)

Each block's, in order.

***

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:973](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L973)

The last block's hash: it commits to every block before it.

***

### step

> **step**: `number`

Defined in: [modules/bitview-client/index.js:974](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L974)

A bucket's width in oracle bins: 0.1, a price range of 0.115%.
