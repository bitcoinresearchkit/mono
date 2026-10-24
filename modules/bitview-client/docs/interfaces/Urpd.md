[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / Urpd

# Interface: Urpd

Defined in: [modules/bitview-client/index.js:1452](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1452)

## Properties

### aggregation

> **aggregation**: [`UrpdAggregation`](../type-aliases/UrpdAggregation.md)

Defined in: [modules/bitview-client/index.js:1458](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1458)

***

### buckets

> **buckets**: [`UrpdBuckets`](UrpdBuckets.md)

Defined in: [modules/bitview-client/index.js:1461](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1461)

***

### close

> **close**: `number`

Defined in: [modules/bitview-client/index.js:1459](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1459)

Price at `height`, in USD. Anchor for `unrealized_pnl`.

***

### cohort

> **cohort**: [`UrpdCohort`](../type-aliases/UrpdCohort.md)

Defined in: [modules/bitview-client/index.js:1456](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1456)

***

### date

> **date**: `string`

Defined in: [modules/bitview-client/index.js:1455](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1455)

UTC date of the represented block's monotonic timestamp.

***

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:1454](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1454)

Hash of block `height`.

***

### height

> **height**: `number`

Defined in: [modules/bitview-client/index.js:1453](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1453)

Exact published block represented by this distribution.

***

### supply

> **supply**: `number`

Defined in: [modules/bitview-client/index.js:1460](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1460)

Sum of `buckets.supply`, in BTC.

***

### weight

> **weight**: [`UrpdWeight`](../type-aliases/UrpdWeight.md)

Defined in: [modules/bitview-client/index.js:1457](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1457)
