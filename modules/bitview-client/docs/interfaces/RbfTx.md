[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / RbfTx

# Interface: RbfTx

Defined in: [modules/bitview-client/index.js:1135](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1135)

## Properties

### fee

> **fee**: `number`

Defined in: [modules/bitview-client/index.js:1137](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1137)

***

### fullRbf?

> `optional` **fullRbf?**: `boolean` \| `null`

Defined in: [modules/bitview-client/index.js:1143](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1143)

Only populated on the root `tx` of an RBF response. `true` iff
this tx displaced at least one non-signaling predecessor.

***

### rate

> **rate**: `number`

Defined in: [modules/bitview-client/index.js:1140](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1140)

***

### rbf

> **rbf**: `boolean`

Defined in: [modules/bitview-client/index.js:1142](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1142)

BIP-125 signaling: at least one input has sequence < 0xffffffff-1.

***

### time

> **time**: `number`

Defined in: [modules/bitview-client/index.js:1141](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1141)

***

### txid

> **txid**: `string`

Defined in: [modules/bitview-client/index.js:1136](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1136)

***

### value

> **value**: `number`

Defined in: [modules/bitview-client/index.js:1139](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1139)

Sum of output amounts.

***

### vsize

> **vsize**: `number`

Defined in: [modules/bitview-client/index.js:1138](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1138)
