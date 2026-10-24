[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / Urpd

# Interface: Urpd

Defined in: [modules/bitview-client/index.js:1513](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1513)

## Properties

### aggregation

> **aggregation**: [`UrpdAggregation`](../type-aliases/UrpdAggregation.md)

Defined in: [modules/bitview-client/index.js:1519](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1519)

***

### buckets

> **buckets**: [`UrpdBuckets`](UrpdBuckets.md)

Defined in: [modules/bitview-client/index.js:1522](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1522)

***

### close

> **close**: `number`

Defined in: [modules/bitview-client/index.js:1520](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1520)

Price at `height`, in USD. Anchor for `unrealized_pnl`.

***

### cohort

> **cohort**: [`UrpdCohort`](../type-aliases/UrpdCohort.md)

Defined in: [modules/bitview-client/index.js:1517](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1517)

***

### date

> **date**: `string`

Defined in: [modules/bitview-client/index.js:1516](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1516)

UTC date of the represented block's monotonic timestamp.

***

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:1515](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1515)

Hash of block `height`.

***

### height

> **height**: `number`

Defined in: [modules/bitview-client/index.js:1514](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1514)

Exact published block represented by this distribution.

***

### supply

> **supply**: `number`

Defined in: [modules/bitview-client/index.js:1521](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1521)

Sum of `buckets.supply`, in BTC.

***

### weight

> **weight**: [`UrpdWeight`](../type-aliases/UrpdWeight.md)

Defined in: [modules/bitview-client/index.js:1518](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1518)
