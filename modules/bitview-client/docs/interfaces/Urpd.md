[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / Urpd

# Interface: Urpd

Defined in: [modules/bitview-client/index.js:1405](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1405)

## Properties

### aggregation

> **aggregation**: [`UrpdAggregation`](../type-aliases/UrpdAggregation.md)

Defined in: [modules/bitview-client/index.js:1411](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1411)

***

### buckets

> **buckets**: [`UrpdBuckets`](UrpdBuckets.md)

Defined in: [modules/bitview-client/index.js:1414](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1414)

***

### close

> **close**: `number`

Defined in: [modules/bitview-client/index.js:1412](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1412)

Price at `height`, in USD. Anchor for `unrealized_pnl`.

***

### cohort

> **cohort**: [`UrpdCohort`](../type-aliases/UrpdCohort.md)

Defined in: [modules/bitview-client/index.js:1409](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1409)

***

### date

> **date**: `string`

Defined in: [modules/bitview-client/index.js:1408](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1408)

UTC date of the represented block's monotonic timestamp.

***

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:1407](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1407)

Hash of block `height`.

***

### height

> **height**: `number`

Defined in: [modules/bitview-client/index.js:1406](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1406)

Exact published block represented by this distribution.

***

### supply

> **supply**: `number`

Defined in: [modules/bitview-client/index.js:1413](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1413)

Sum of `buckets.supply`, in BTC.

***

### weight

> **weight**: [`UrpdWeight`](../type-aliases/UrpdWeight.md)

Defined in: [modules/bitview-client/index.js:1410](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1410)
