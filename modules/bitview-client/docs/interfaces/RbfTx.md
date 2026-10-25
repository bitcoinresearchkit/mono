[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / RbfTx

# Interface: RbfTx

Defined in: [modules/bitview-client/index.js:1249](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1249)

## Properties

### fee

> **fee**: `number`

Defined in: [modules/bitview-client/index.js:1251](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1251)

***

### fullRbf?

> `optional` **fullRbf?**: `boolean` \| `null`

Defined in: [modules/bitview-client/index.js:1257](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1257)

Only populated on the root `tx` of an RBF response. `true` iff
this tx displaced at least one non-signaling predecessor.

***

### rate

> **rate**: `number`

Defined in: [modules/bitview-client/index.js:1254](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1254)

***

### rbf

> **rbf**: `boolean`

Defined in: [modules/bitview-client/index.js:1256](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1256)

BIP-125 signaling: at least one input has sequence < 0xffffffff-1.

***

### time

> **time**: `number`

Defined in: [modules/bitview-client/index.js:1255](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1255)

***

### txid

> **txid**: `string`

Defined in: [modules/bitview-client/index.js:1250](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1250)

***

### value

> **value**: `number`

Defined in: [modules/bitview-client/index.js:1253](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1253)

Sum of output amounts.

***

### vsize

> **vsize**: `number`

Defined in: [modules/bitview-client/index.js:1252](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1252)
