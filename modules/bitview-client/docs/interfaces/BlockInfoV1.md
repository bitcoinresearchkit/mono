[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockInfoV1

# Interface: BlockInfoV1

Defined in: [modules/bitview-client/index.js:243](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L243)

## Properties

### bits

> **bits**: `number`

Defined in: [modules/bitview-client/index.js:248](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L248)

Compact target (bits)

***

### difficulty

> **difficulty**: `number`

Defined in: [modules/bitview-client/index.js:250](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L250)

***

### extras

> **extras**: [`BlockExtras`](BlockExtras.md)

Defined in: [modules/bitview-client/index.js:258](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L258)

***

### height

> **height**: `number`

Defined in: [modules/bitview-client/index.js:245](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L245)

***

### id

> **id**: `string`

Defined in: [modules/bitview-client/index.js:244](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L244)

Block hash

***

### mediantime

> **mediantime**: `number`

Defined in: [modules/bitview-client/index.js:256](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L256)

Median time of the last 11 blocks

***

### merkleRoot

> **merkleRoot**: `string`

Defined in: [modules/bitview-client/index.js:251](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L251)

***

### nonce

> **nonce**: `number`

Defined in: [modules/bitview-client/index.js:249](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L249)

***

### previousblockhash?

> `optional` **previousblockhash?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:255](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L255)

Hash of the previous block (null for the genesis block)

***

### size

> **size**: `number`

Defined in: [modules/bitview-client/index.js:253](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L253)

Block size in bytes

***

### stale?

> `optional` **stale?**: `boolean`

Defined in: [modules/bitview-client/index.js:257](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L257)

Whether this block has been replaced by a longer chain

***

### timestamp

> **timestamp**: `number`

Defined in: [modules/bitview-client/index.js:247](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L247)

Block timestamp (Unix time)

***

### txCount

> **txCount**: `number`

Defined in: [modules/bitview-client/index.js:252](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L252)

***

### version

> **version**: `number`

Defined in: [modules/bitview-client/index.js:246](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L246)

***

### weight

> **weight**: `number`

Defined in: [modules/bitview-client/index.js:254](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L254)

Block weight in weight units
