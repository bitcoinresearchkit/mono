[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / RbfTx

# Interface: RbfTx

Defined in: [modules/bitview-client/index.js:1181](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1181)

## Properties

### fee

> **fee**: `number`

Defined in: [modules/bitview-client/index.js:1183](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1183)

***

### fullRbf?

> `optional` **fullRbf?**: `boolean` \| `null`

Defined in: [modules/bitview-client/index.js:1189](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1189)

Only populated on the root `tx` of an RBF response. `true` iff
this tx displaced at least one non-signaling predecessor.

***

### rate

> **rate**: `number`

Defined in: [modules/bitview-client/index.js:1186](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1186)

***

### rbf

> **rbf**: `boolean`

Defined in: [modules/bitview-client/index.js:1188](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1188)

BIP-125 signaling: at least one input has sequence < 0xffffffff-1.

***

### time

> **time**: `number`

Defined in: [modules/bitview-client/index.js:1187](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1187)

***

### txid

> **txid**: `string`

Defined in: [modules/bitview-client/index.js:1182](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1182)

***

### value

> **value**: `number`

Defined in: [modules/bitview-client/index.js:1185](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1185)

Sum of output amounts.

***

### vsize

> **vsize**: `number`

Defined in: [modules/bitview-client/index.js:1184](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1184)
