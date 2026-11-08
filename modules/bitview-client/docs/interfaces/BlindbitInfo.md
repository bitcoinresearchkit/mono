[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlindbitInfo

# Interface: BlindbitInfo

Defined in: [modules/bitview-client/index.js:105](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L105)

## Properties

### height

> **height**: `number`

Defined in: [modules/bitview-client/index.js:107](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L107)

The last indexed block.

***

### network

> **network**: `string`

Defined in: [modules/bitview-client/index.js:106](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L106)

Bitcoin Core's chain name: `main`.

***

### tweaksCutThroughWithDustFilter

> **tweaksCutThroughWithDustFilter**: `boolean`

Defined in: [modules/bitview-client/index.js:111](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L111)

Whether cut-through tweaks (of transactions with an unspent Taproot output) take a dust
  limit: `true`.

***

### tweaksFullBasic

> **tweaksFullBasic**: `boolean`

Defined in: [modules/bitview-client/index.js:109](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L109)

Whether the full tweak index is served without dust limits: `false`, it takes them.

***

### tweaksFullWithDustFilter

> **tweaksFullWithDustFilter**: `boolean`

Defined in: [modules/bitview-client/index.js:110](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L110)

Whether the full tweak index takes a dust limit: `true`.

***

### tweaksOnly

> **tweaksOnly**: `boolean`

Defined in: [modules/bitview-client/index.js:108](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L108)

Whether only tweaks are served: `false`, filters and UTXOs are too.
