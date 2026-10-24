[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockInfoV1

# Interface: BlockInfoV1

Defined in: [modules/bitview-client/index.js:217](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L217)

## Properties

### bits

> **bits**: `number`

Defined in: [modules/bitview-client/index.js:222](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L222)

Compact target (bits)

***

### difficulty

> **difficulty**: `number`

Defined in: [modules/bitview-client/index.js:224](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L224)

***

### extras

> **extras**: [`BlockExtras`](BlockExtras.md)

Defined in: [modules/bitview-client/index.js:232](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L232)

***

### height

> **height**: `number`

Defined in: [modules/bitview-client/index.js:219](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L219)

***

### id

> **id**: `string`

Defined in: [modules/bitview-client/index.js:218](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L218)

Block hash

***

### mediantime

> **mediantime**: `number`

Defined in: [modules/bitview-client/index.js:230](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L230)

Median time of the last 11 blocks

***

### merkleRoot

> **merkleRoot**: `string`

Defined in: [modules/bitview-client/index.js:225](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L225)

***

### nonce

> **nonce**: `number`

Defined in: [modules/bitview-client/index.js:223](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L223)

***

### previousblockhash?

> `optional` **previousblockhash?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:229](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L229)

Hash of the previous block (null for the genesis block)

***

### size

> **size**: `number`

Defined in: [modules/bitview-client/index.js:227](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L227)

Block size in bytes

***

### stale?

> `optional` **stale?**: `boolean`

Defined in: [modules/bitview-client/index.js:231](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L231)

Whether this block has been replaced by a longer chain

***

### timestamp

> **timestamp**: `number`

Defined in: [modules/bitview-client/index.js:221](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L221)

Block timestamp (Unix time)

***

### txCount

> **txCount**: `number`

Defined in: [modules/bitview-client/index.js:226](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L226)

***

### version

> **version**: `number`

Defined in: [modules/bitview-client/index.js:220](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L220)

***

### weight

> **weight**: `number`

Defined in: [modules/bitview-client/index.js:228](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L228)

Block weight in weight units
