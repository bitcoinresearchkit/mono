[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / RbfTx

# Interface: RbfTx

Defined in: [modules/bitview-client/index.js:1063](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1063)

## Properties

### fee

> **fee**: `number`

Defined in: [modules/bitview-client/index.js:1065](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1065)

***

### fullRbf?

> `optional` **fullRbf?**: `boolean` \| `null`

Defined in: [modules/bitview-client/index.js:1071](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1071)

Only populated on the root `tx` of an RBF response. `true` iff
this tx displaced at least one non-signaling predecessor.

***

### rate

> **rate**: `number`

Defined in: [modules/bitview-client/index.js:1068](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1068)

***

### rbf

> **rbf**: `boolean`

Defined in: [modules/bitview-client/index.js:1070](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1070)

BIP-125 signaling: at least one input has sequence < 0xffffffff-1.

***

### time

> **time**: `number`

Defined in: [modules/bitview-client/index.js:1069](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1069)

***

### txid

> **txid**: `string`

Defined in: [modules/bitview-client/index.js:1064](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1064)

***

### value

> **value**: `number`

Defined in: [modules/bitview-client/index.js:1067](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1067)

Sum of output amounts.

***

### vsize

> **vsize**: `number`

Defined in: [modules/bitview-client/index.js:1066](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1066)
