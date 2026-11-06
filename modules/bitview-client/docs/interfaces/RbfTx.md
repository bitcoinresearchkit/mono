[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / RbfTx

# Interface: RbfTx

Defined in: [modules/bitview-client/index.js:1259](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1259)

## Properties

### fee

> **fee**: `number`

Defined in: [modules/bitview-client/index.js:1261](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1261)

***

### fullRbf?

> `optional` **fullRbf?**: `boolean` \| `null`

Defined in: [modules/bitview-client/index.js:1267](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1267)

Only populated on the root `tx` of an RBF response. `true` iff
  this tx displaced at least one non-signaling predecessor.

***

### rate

> **rate**: `number`

Defined in: [modules/bitview-client/index.js:1264](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1264)

***

### rbf

> **rbf**: `boolean`

Defined in: [modules/bitview-client/index.js:1266](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1266)

BIP-125 signaling: at least one input has sequence < 0xffffffff-1.

***

### time

> **time**: `number`

Defined in: [modules/bitview-client/index.js:1265](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1265)

***

### txid

> **txid**: `string`

Defined in: [modules/bitview-client/index.js:1260](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1260)

***

### value

> **value**: `number`

Defined in: [modules/bitview-client/index.js:1263](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1263)

Sum of output amounts.

***

### vsize

> **vsize**: `number`

Defined in: [modules/bitview-client/index.js:1262](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1262)
