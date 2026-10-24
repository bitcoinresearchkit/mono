[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / RbfTx

# Interface: RbfTx

Defined in: [modules/bitview-client/index.js:1157](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1157)

## Properties

### fee

> **fee**: `number`

Defined in: [modules/bitview-client/index.js:1159](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1159)

***

### fullRbf?

> `optional` **fullRbf?**: `boolean` \| `null`

Defined in: [modules/bitview-client/index.js:1165](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1165)

Only populated on the root `tx` of an RBF response. `true` iff
this tx displaced at least one non-signaling predecessor.

***

### rate

> **rate**: `number`

Defined in: [modules/bitview-client/index.js:1162](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1162)

***

### rbf

> **rbf**: `boolean`

Defined in: [modules/bitview-client/index.js:1164](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1164)

BIP-125 signaling: at least one input has sequence < 0xffffffff-1.

***

### time

> **time**: `number`

Defined in: [modules/bitview-client/index.js:1163](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1163)

***

### txid

> **txid**: `string`

Defined in: [modules/bitview-client/index.js:1158](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1158)

***

### value

> **value**: `number`

Defined in: [modules/bitview-client/index.js:1161](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1161)

Sum of output amounts.

***

### vsize

> **vsize**: `number`

Defined in: [modules/bitview-client/index.js:1160](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1160)
