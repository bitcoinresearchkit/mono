[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / RbfTx

# Interface: RbfTx

Defined in: [Developer/mono/modules/bitview-client/index.js:1147](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1147)

## Properties

### fee

> **fee**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1149](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1149)

***

### fullRbf?

> `optional` **fullRbf?**: `boolean` \| `null`

Defined in: [Developer/mono/modules/bitview-client/index.js:1155](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1155)

Only populated on the root `tx` of an RBF response. `true` iff
this tx displaced at least one non-signaling predecessor.

***

### rate

> **rate**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1152](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1152)

***

### rbf

> **rbf**: `boolean`

Defined in: [Developer/mono/modules/bitview-client/index.js:1154](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1154)

BIP-125 signaling: at least one input has sequence < 0xffffffff-1.

***

### time

> **time**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1153](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1153)

***

### txid

> **txid**: `string`

Defined in: [Developer/mono/modules/bitview-client/index.js:1148](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1148)

***

### value

> **value**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1151](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1151)

Sum of output amounts.

***

### vsize

> **vsize**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1150](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1150)
