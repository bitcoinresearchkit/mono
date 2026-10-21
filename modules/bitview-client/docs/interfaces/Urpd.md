[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / Urpd

# Interface: Urpd

Defined in: [modules/bitview-client/index.js:1362](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1362)

## Properties

### aggregation

> **aggregation**: [`UrpdAggregation`](../type-aliases/UrpdAggregation.md)

Defined in: [modules/bitview-client/index.js:1368](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1368)

***

### buckets

> **buckets**: [`UrpdBuckets`](UrpdBuckets.md)

Defined in: [modules/bitview-client/index.js:1371](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1371)

***

### close

> **close**: `number`

Defined in: [modules/bitview-client/index.js:1369](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1369)

Price at `height`, in USD. Anchor for `unrealized_pnl`.

***

### cohort

> **cohort**: [`UrpdCohort`](../type-aliases/UrpdCohort.md)

Defined in: [modules/bitview-client/index.js:1366](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1366)

***

### date

> **date**: `string`

Defined in: [modules/bitview-client/index.js:1365](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1365)

UTC date of the represented block's monotonic timestamp.

***

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:1364](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1364)

Hash of block `height`.

***

### height

> **height**: `number`

Defined in: [modules/bitview-client/index.js:1363](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1363)

Exact published block represented by this distribution.

***

### supply

> **supply**: `number`

Defined in: [modules/bitview-client/index.js:1370](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1370)

Sum of `buckets.supply`, in BTC.

***

### weight

> **weight**: [`UrpdWeight`](../type-aliases/UrpdWeight.md)

Defined in: [modules/bitview-client/index.js:1367](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1367)
