[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / RbfTx

# Interface: RbfTx

Defined in: [modules/bitview-client/index.js:1213](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1213)

## Properties

### fee

> **fee**: `number`

Defined in: [modules/bitview-client/index.js:1215](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1215)

***

### fullRbf?

> `optional` **fullRbf?**: `boolean` \| `null`

Defined in: [modules/bitview-client/index.js:1221](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1221)

Only populated on the root `tx` of an RBF response. `true` iff
this tx displaced at least one non-signaling predecessor.

***

### rate

> **rate**: `number`

Defined in: [modules/bitview-client/index.js:1218](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1218)

***

### rbf

> **rbf**: `boolean`

Defined in: [modules/bitview-client/index.js:1220](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1220)

BIP-125 signaling: at least one input has sequence < 0xffffffff-1.

***

### time

> **time**: `number`

Defined in: [modules/bitview-client/index.js:1219](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1219)

***

### txid

> **txid**: `string`

Defined in: [modules/bitview-client/index.js:1214](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1214)

***

### value

> **value**: `number`

Defined in: [modules/bitview-client/index.js:1217](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1217)

Sum of output amounts.

***

### vsize

> **vsize**: `number`

Defined in: [modules/bitview-client/index.js:1216](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1216)
