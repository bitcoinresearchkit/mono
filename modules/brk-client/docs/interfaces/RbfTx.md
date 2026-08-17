[**brk-client**](../README.md)

***

[brk-client](../globals.md) / RbfTx

# Interface: RbfTx

Defined in: [Developer/brk/modules/brk-client/index.js:1097](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L1097)

## Properties

### fee

> **fee**: `number`

Defined in: [Developer/brk/modules/brk-client/index.js:1099](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L1099)

***

### fullRbf?

> `optional` **fullRbf?**: `boolean` \| `null`

Defined in: [Developer/brk/modules/brk-client/index.js:1105](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L1105)

Only populated on the root `tx` of an RBF response. `true` iff
this tx displaced at least one non-signaling predecessor.

***

### rate

> **rate**: `number`

Defined in: [Developer/brk/modules/brk-client/index.js:1102](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L1102)

***

### rbf

> **rbf**: `boolean`

Defined in: [Developer/brk/modules/brk-client/index.js:1104](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L1104)

BIP-125 signaling: at least one input has sequence < 0xffffffff-1.

***

### time

> **time**: `number`

Defined in: [Developer/brk/modules/brk-client/index.js:1103](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L1103)

***

### txid

> **txid**: `string`

Defined in: [Developer/brk/modules/brk-client/index.js:1098](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L1098)

***

### value

> **value**: `number`

Defined in: [Developer/brk/modules/brk-client/index.js:1101](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L1101)

Sum of output amounts.

***

### vsize

> **vsize**: `number`

Defined in: [Developer/brk/modules/brk-client/index.js:1100](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L1100)
