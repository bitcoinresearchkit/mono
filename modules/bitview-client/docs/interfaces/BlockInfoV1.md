[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockInfoV1

# Interface: BlockInfoV1

Defined in: [modules/bitview-client/index.js:232](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L232)

## Properties

### bits

> **bits**: `number`

Defined in: [modules/bitview-client/index.js:237](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L237)

Compact target (bits)

***

### difficulty

> **difficulty**: `number`

Defined in: [modules/bitview-client/index.js:239](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L239)

***

### extras

> **extras**: [`BlockExtras`](BlockExtras.md)

Defined in: [modules/bitview-client/index.js:247](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L247)

***

### height

> **height**: `number`

Defined in: [modules/bitview-client/index.js:234](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L234)

***

### id

> **id**: `string`

Defined in: [modules/bitview-client/index.js:233](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L233)

Block hash

***

### mediantime

> **mediantime**: `number`

Defined in: [modules/bitview-client/index.js:245](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L245)

Median time of the last 11 blocks

***

### merkleRoot

> **merkleRoot**: `string`

Defined in: [modules/bitview-client/index.js:240](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L240)

***

### nonce

> **nonce**: `number`

Defined in: [modules/bitview-client/index.js:238](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L238)

***

### previousblockhash?

> `optional` **previousblockhash?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:244](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L244)

Hash of the previous block (null for the genesis block)

***

### size

> **size**: `number`

Defined in: [modules/bitview-client/index.js:242](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L242)

Block size in bytes

***

### stale?

> `optional` **stale?**: `boolean`

Defined in: [modules/bitview-client/index.js:246](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L246)

Whether this block has been replaced by a longer chain

***

### timestamp

> **timestamp**: `number`

Defined in: [modules/bitview-client/index.js:236](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L236)

Block timestamp (Unix time)

***

### txCount

> **txCount**: `number`

Defined in: [modules/bitview-client/index.js:241](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L241)

***

### version

> **version**: `number`

Defined in: [modules/bitview-client/index.js:235](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L235)

***

### weight

> **weight**: `number`

Defined in: [modules/bitview-client/index.js:243](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L243)

Block weight in weight units
