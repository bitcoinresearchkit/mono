[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / RbfTx

# Interface: RbfTx

Defined in: modules/bitview-client/index.js:1251

## Properties

### fee

> **fee**: `number`

Defined in: modules/bitview-client/index.js:1253

***

### fullRbf?

> `optional` **fullRbf?**: `boolean` \| `null`

Defined in: modules/bitview-client/index.js:1259

Only populated on the root `tx` of an RBF response. `true` iff
this tx displaced at least one non-signaling predecessor.

***

### rate

> **rate**: `number`

Defined in: modules/bitview-client/index.js:1256

***

### rbf

> **rbf**: `boolean`

Defined in: modules/bitview-client/index.js:1258

BIP-125 signaling: at least one input has sequence < 0xffffffff-1.

***

### time

> **time**: `number`

Defined in: modules/bitview-client/index.js:1257

***

### txid

> **txid**: `string`

Defined in: modules/bitview-client/index.js:1252

***

### value

> **value**: `number`

Defined in: modules/bitview-client/index.js:1255

Sum of output amounts.

***

### vsize

> **vsize**: `number`

Defined in: modules/bitview-client/index.js:1254
