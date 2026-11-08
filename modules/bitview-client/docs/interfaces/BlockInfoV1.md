[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockInfoV1

# Interface: BlockInfoV1

Defined in: [modules/bitview-client/index.js:248](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L248)

## Properties

### bits

> **bits**: `number`

Defined in: [modules/bitview-client/index.js:253](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L253)

Compact target (bits)

***

### difficulty

> **difficulty**: `number`

Defined in: [modules/bitview-client/index.js:255](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L255)

***

### extras

> **extras**: [`BlockExtras`](BlockExtras.md)

Defined in: [modules/bitview-client/index.js:263](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L263)

***

### height

> **height**: `number`

Defined in: [modules/bitview-client/index.js:250](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L250)

***

### id

> **id**: `string`

Defined in: [modules/bitview-client/index.js:249](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L249)

Block hash

***

### mediantime

> **mediantime**: `number`

Defined in: [modules/bitview-client/index.js:261](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L261)

Median time of the last 11 blocks

***

### merkleRoot

> **merkleRoot**: `string`

Defined in: [modules/bitview-client/index.js:256](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L256)

***

### nonce

> **nonce**: `number`

Defined in: [modules/bitview-client/index.js:254](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L254)

***

### previousblockhash?

> `optional` **previousblockhash?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:260](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L260)

Hash of the previous block (null for the genesis block)

***

### size

> **size**: `number`

Defined in: [modules/bitview-client/index.js:258](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L258)

Block size in bytes

***

### stale?

> `optional` **stale?**: `boolean`

Defined in: [modules/bitview-client/index.js:262](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L262)

Whether this block has been replaced by a longer chain

***

### timestamp

> **timestamp**: `number`

Defined in: [modules/bitview-client/index.js:252](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L252)

Block timestamp (Unix time)

***

### txCount

> **txCount**: `number`

Defined in: [modules/bitview-client/index.js:257](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L257)

***

### version

> **version**: `number`

Defined in: [modules/bitview-client/index.js:251](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L251)

***

### weight

> **weight**: `number`

Defined in: [modules/bitview-client/index.js:259](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L259)

Block weight in weight units
