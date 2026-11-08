[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / RbfTx

# Interface: RbfTx

Defined in: [modules/bitview-client/index.js:1219](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1219)

## Properties

### fee

> **fee**: `number`

Defined in: [modules/bitview-client/index.js:1221](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1221)

***

### fullRbf?

> `optional` **fullRbf?**: `boolean` \| `null`

Defined in: [modules/bitview-client/index.js:1227](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1227)

On a transaction that replaced others: whether one it directly replaced didn't signal
  (mempool.space's `tx.fullRbf`). Absent on one that replaced none.

***

### mined?

> `optional` **mined?**: `boolean` \| `null`

Defined in: [modules/bitview-client/index.js:1229](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1229)

`true` when this transaction is confirmed; absent otherwise.

***

### rate

> **rate**: `number`

Defined in: [modules/bitview-client/index.js:1224](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1224)

***

### rbf

> **rbf**: `boolean`

Defined in: [modules/bitview-client/index.js:1226](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1226)

BIP-125 signaling: at least one input has sequence < 0xffffffff-1.

***

### time

> **time**: `number`

Defined in: [modules/bitview-client/index.js:1225](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1225)

***

### txid

> **txid**: `string`

Defined in: [modules/bitview-client/index.js:1220](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1220)

***

### value

> **value**: `number`

Defined in: [modules/bitview-client/index.js:1223](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1223)

Sum of output amounts.

***

### vsize

> **vsize**: `number`

Defined in: [modules/bitview-client/index.js:1222](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1222)
