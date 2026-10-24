[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / RbfTx

# Interface: RbfTx

Defined in: [modules/bitview-client/index.js:1163](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1163)

## Properties

### fee

> **fee**: `number`

Defined in: [modules/bitview-client/index.js:1165](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1165)

***

### fullRbf?

> `optional` **fullRbf?**: `boolean` \| `null`

Defined in: [modules/bitview-client/index.js:1171](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1171)

Only populated on the root `tx` of an RBF response. `true` iff
this tx displaced at least one non-signaling predecessor.

***

### rate

> **rate**: `number`

Defined in: [modules/bitview-client/index.js:1168](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1168)

***

### rbf

> **rbf**: `boolean`

Defined in: [modules/bitview-client/index.js:1170](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1170)

BIP-125 signaling: at least one input has sequence < 0xffffffff-1.

***

### time

> **time**: `number`

Defined in: [modules/bitview-client/index.js:1169](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1169)

***

### txid

> **txid**: `string`

Defined in: [modules/bitview-client/index.js:1164](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1164)

***

### value

> **value**: `number`

Defined in: [modules/bitview-client/index.js:1167](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1167)

Sum of output amounts.

***

### vsize

> **vsize**: `number`

Defined in: [modules/bitview-client/index.js:1166](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1166)
