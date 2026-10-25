[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / RbfTx

# Interface: RbfTx

Defined in: [modules/bitview-client/index.js:1231](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1231)

## Properties

### fee

> **fee**: `number`

Defined in: [modules/bitview-client/index.js:1233](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1233)

***

### fullRbf?

> `optional` **fullRbf?**: `boolean` \| `null`

Defined in: [modules/bitview-client/index.js:1239](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1239)

Only populated on the root `tx` of an RBF response. `true` iff
this tx displaced at least one non-signaling predecessor.

***

### rate

> **rate**: `number`

Defined in: [modules/bitview-client/index.js:1236](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1236)

***

### rbf

> **rbf**: `boolean`

Defined in: [modules/bitview-client/index.js:1238](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1238)

BIP-125 signaling: at least one input has sequence < 0xffffffff-1.

***

### time

> **time**: `number`

Defined in: [modules/bitview-client/index.js:1237](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1237)

***

### txid

> **txid**: `string`

Defined in: [modules/bitview-client/index.js:1232](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1232)

***

### value

> **value**: `number`

Defined in: [modules/bitview-client/index.js:1235](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1235)

Sum of output amounts.

***

### vsize

> **vsize**: `number`

Defined in: [modules/bitview-client/index.js:1234](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1234)
