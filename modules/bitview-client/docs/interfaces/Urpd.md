[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / Urpd

# Interface: Urpd

Defined in: [modules/bitview-client/index.js:1553](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1553)

## Properties

### aggregation

> **aggregation**: [`UrpdAggregation`](../type-aliases/UrpdAggregation.md)

Defined in: [modules/bitview-client/index.js:1559](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1559)

***

### buckets

> **buckets**: [`UrpdBuckets`](UrpdBuckets.md)

Defined in: [modules/bitview-client/index.js:1562](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1562)

***

### close

> **close**: `number`

Defined in: [modules/bitview-client/index.js:1560](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1560)

Price at `height`, in USD. Anchor for `unrealized_pnl`.

***

### cohort

> **cohort**: [`UrpdCohort`](../type-aliases/UrpdCohort.md)

Defined in: [modules/bitview-client/index.js:1557](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1557)

***

### date

> **date**: `string`

Defined in: [modules/bitview-client/index.js:1556](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1556)

UTC date of the represented block's monotonic timestamp.

***

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:1555](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1555)

Hash of block `height`.

***

### height

> **height**: `number`

Defined in: [modules/bitview-client/index.js:1554](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1554)

Exact published block represented by this distribution.

***

### supply

> **supply**: `number`

Defined in: [modules/bitview-client/index.js:1561](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1561)

Sum of `buckets.supply`, in BTC.

***

### weight

> **weight**: [`UrpdWeight`](../type-aliases/UrpdWeight.md)

Defined in: [modules/bitview-client/index.js:1558](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1558)
