[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / RbfTx

# Interface: RbfTx

Defined in: [Developer/mono/modules/bitview-client/index.js:1027](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1027)

## Properties

### fee

> **fee**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1029](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1029)

***

### fullRbf?

> `optional` **fullRbf?**: `boolean` \| `null`

Defined in: [Developer/mono/modules/bitview-client/index.js:1035](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1035)

Only populated on the root `tx` of an RBF response. `true` iff
this tx displaced at least one non-signaling predecessor.

***

### rate

> **rate**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1032](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1032)

***

### rbf

> **rbf**: `boolean`

Defined in: [Developer/mono/modules/bitview-client/index.js:1034](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1034)

BIP-125 signaling: at least one input has sequence < 0xffffffff-1.

***

### time

> **time**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1033](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1033)

***

### txid

> **txid**: `string`

Defined in: [Developer/mono/modules/bitview-client/index.js:1028](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1028)

***

### value

> **value**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1031](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1031)

Sum of output amounts.

***

### vsize

> **vsize**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1030](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1030)
