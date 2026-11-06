[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockInfoV1

# Interface: BlockInfoV1

Defined in: [modules/bitview-client/index.js:190](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L190)

## Properties

### bits

> **bits**: `number`

Defined in: [modules/bitview-client/index.js:195](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L195)

Compact target (bits)

***

### difficulty

> **difficulty**: `number`

Defined in: [modules/bitview-client/index.js:197](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L197)

***

### extras

> **extras**: [`BlockExtras`](BlockExtras.md)

Defined in: [modules/bitview-client/index.js:205](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L205)

***

### height

> **height**: `number`

Defined in: [modules/bitview-client/index.js:192](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L192)

***

### id

> **id**: `string`

Defined in: [modules/bitview-client/index.js:191](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L191)

Block hash

***

### mediantime

> **mediantime**: `number`

Defined in: [modules/bitview-client/index.js:203](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L203)

Median time of the last 11 blocks

***

### merkleRoot

> **merkleRoot**: `string`

Defined in: [modules/bitview-client/index.js:198](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L198)

***

### nonce

> **nonce**: `number`

Defined in: [modules/bitview-client/index.js:196](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L196)

***

### previousblockhash?

> `optional` **previousblockhash?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:202](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L202)

Hash of the previous block (null for the genesis block)

***

### size

> **size**: `number`

Defined in: [modules/bitview-client/index.js:200](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L200)

Block size in bytes

***

### stale?

> `optional` **stale?**: `boolean`

Defined in: [modules/bitview-client/index.js:204](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L204)

Whether this block has been replaced by a longer chain

***

### timestamp

> **timestamp**: `number`

Defined in: [modules/bitview-client/index.js:194](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L194)

Block timestamp (Unix time)

***

### txCount

> **txCount**: `number`

Defined in: [modules/bitview-client/index.js:199](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L199)

***

### version

> **version**: `number`

Defined in: [modules/bitview-client/index.js:193](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L193)

***

### weight

> **weight**: `number`

Defined in: [modules/bitview-client/index.js:201](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L201)

Block weight in weight units
