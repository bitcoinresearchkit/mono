[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / RbfTx

# Interface: RbfTx

Defined in: [modules/bitview-client/index.js:1068](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1068)

## Properties

### fee

> **fee**: `number`

Defined in: [modules/bitview-client/index.js:1070](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1070)

***

### fullRbf?

> `optional` **fullRbf?**: `boolean` \| `null`

Defined in: [modules/bitview-client/index.js:1076](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1076)

Only populated on the root `tx` of an RBF response. `true` iff
this tx displaced at least one non-signaling predecessor.

***

### rate

> **rate**: `number`

Defined in: [modules/bitview-client/index.js:1073](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1073)

***

### rbf

> **rbf**: `boolean`

Defined in: [modules/bitview-client/index.js:1075](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1075)

BIP-125 signaling: at least one input has sequence < 0xffffffff-1.

***

### time

> **time**: `number`

Defined in: [modules/bitview-client/index.js:1074](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1074)

***

### txid

> **txid**: `string`

Defined in: [modules/bitview-client/index.js:1069](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1069)

***

### value

> **value**: `number`

Defined in: [modules/bitview-client/index.js:1072](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1072)

Sum of output amounts.

***

### vsize

> **vsize**: `number`

Defined in: [modules/bitview-client/index.js:1071](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1071)
