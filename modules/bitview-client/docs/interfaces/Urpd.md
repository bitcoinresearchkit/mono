[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / Urpd

# Interface: Urpd

Defined in: [Developer/mono/modules/bitview-client/index.js:1374](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1374)

## Properties

### aggregation

> **aggregation**: [`UrpdAggregation`](../type-aliases/UrpdAggregation.md)

Defined in: [Developer/mono/modules/bitview-client/index.js:1379](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1379)

Aggregation strategy applied to the buckets.

***

### buckets

> **buckets**: [`UrpdBucket`](UrpdBucket.md)[]

Defined in: [Developer/mono/modules/bitview-client/index.js:1382](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1382)

***

### close

> **close**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1380](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1380)

Price at `height`, in USD. Anchor for `unrealized_pnl`.

***

### cohort

> **cohort**: `string`

Defined in: [Developer/mono/modules/bitview-client/index.js:1375](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1375)

***

### date

> **date**: `string`

Defined in: [Developer/mono/modules/bitview-client/index.js:1376](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1376)

UTC date of the represented block.

***

### height

> **height**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1377](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1377)

Exact published block represented by this distribution.

***

### totalSupply

> **totalSupply**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1381](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1381)

Sum of `supply` across all buckets, in BTC.

***

### weight

> **weight**: [`UrpdWeight`](../type-aliases/UrpdWeight.md)

Defined in: [Developer/mono/modules/bitview-client/index.js:1378](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1378)

Weighting applied to the source supply.
