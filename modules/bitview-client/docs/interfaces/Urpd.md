[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / Urpd

# Interface: Urpd

Defined in: [modules/bitview-client/index.js:1429](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1429)

## Properties

### aggregation

> **aggregation**: [`UrpdAggregation`](../type-aliases/UrpdAggregation.md)

Defined in: [modules/bitview-client/index.js:1435](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1435)

***

### buckets

> **buckets**: [`UrpdBuckets`](UrpdBuckets.md)

Defined in: [modules/bitview-client/index.js:1438](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1438)

***

### close

> **close**: `number`

Defined in: [modules/bitview-client/index.js:1436](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1436)

Price at `height`, in USD. Anchor for `unrealized_pnl`.

***

### cohort

> **cohort**: [`UrpdCohort`](../type-aliases/UrpdCohort.md)

Defined in: [modules/bitview-client/index.js:1433](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1433)

***

### date

> **date**: `string`

Defined in: [modules/bitview-client/index.js:1432](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1432)

UTC date of the represented block's monotonic timestamp.

***

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:1431](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1431)

Hash of block `height`.

***

### height

> **height**: `number`

Defined in: [modules/bitview-client/index.js:1430](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1430)

Exact published block represented by this distribution.

***

### supply

> **supply**: `number`

Defined in: [modules/bitview-client/index.js:1437](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1437)

Sum of `buckets.supply`, in BTC.

***

### weight

> **weight**: [`UrpdWeight`](../type-aliases/UrpdWeight.md)

Defined in: [modules/bitview-client/index.js:1434](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1434)
