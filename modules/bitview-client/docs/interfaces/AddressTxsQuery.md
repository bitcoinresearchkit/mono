[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / AddressTxsQuery

# Interface: AddressTxsQuery

Defined in: [modules/bitview-client/index.js:83](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L83)

## Properties

### afterTxid?

> `optional` **afterTxid?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:84](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L84)

Continue after this transaction, a mempool or a confirmed one: the older ones (a confirmed
cursor skips the mempool). One that is neither answers 422.

***

### maxTxs?

> `optional` **maxTxs?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:86](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L86)

Transactions to return, at most 50 (the default); larger values return 50.
