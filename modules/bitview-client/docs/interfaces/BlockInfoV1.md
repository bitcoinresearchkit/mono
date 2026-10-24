[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockInfoV1

# Interface: BlockInfoV1

Defined in: [modules/bitview-client/index.js:237](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L237)

## Properties

### bits

> **bits**: `number`

Defined in: [modules/bitview-client/index.js:242](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L242)

Compact target (bits)

***

### difficulty

> **difficulty**: `number`

Defined in: [modules/bitview-client/index.js:244](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L244)

***

### extras

> **extras**: [`BlockExtras`](BlockExtras.md)

Defined in: [modules/bitview-client/index.js:252](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L252)

***

### height

> **height**: `number`

Defined in: [modules/bitview-client/index.js:239](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L239)

***

### id

> **id**: `string`

Defined in: [modules/bitview-client/index.js:238](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L238)

Block hash

***

### mediantime

> **mediantime**: `number`

Defined in: [modules/bitview-client/index.js:250](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L250)

Median time of the last 11 blocks

***

### merkleRoot

> **merkleRoot**: `string`

Defined in: [modules/bitview-client/index.js:245](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L245)

***

### nonce

> **nonce**: `number`

Defined in: [modules/bitview-client/index.js:243](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L243)

***

### previousblockhash?

> `optional` **previousblockhash?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:249](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L249)

Hash of the previous block (null for the genesis block)

***

### size

> **size**: `number`

Defined in: [modules/bitview-client/index.js:247](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L247)

Block size in bytes

***

### stale?

> `optional` **stale?**: `boolean`

Defined in: [modules/bitview-client/index.js:251](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L251)

Whether this block has been replaced by a longer chain

***

### timestamp

> **timestamp**: `number`

Defined in: [modules/bitview-client/index.js:241](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L241)

Block timestamp (Unix time)

***

### txCount

> **txCount**: `number`

Defined in: [modules/bitview-client/index.js:246](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L246)

***

### version

> **version**: `number`

Defined in: [modules/bitview-client/index.js:240](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L240)

***

### weight

> **weight**: `number`

Defined in: [modules/bitview-client/index.js:248](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L248)

Block weight in weight units
