[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / Urpd

# Interface: Urpd

Defined in: [modules/bitview-client/index.js:1457](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1457)

## Properties

### aggregation

> **aggregation**: [`UrpdAggregation`](../type-aliases/UrpdAggregation.md)

Defined in: [modules/bitview-client/index.js:1463](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1463)

***

### buckets

> **buckets**: [`UrpdBuckets`](UrpdBuckets.md)

Defined in: [modules/bitview-client/index.js:1466](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1466)

***

### close

> **close**: `number`

Defined in: [modules/bitview-client/index.js:1464](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1464)

Price at `height`, in USD. Anchor for `unrealized_pnl`.

***

### cohort

> **cohort**: [`UrpdCohort`](../type-aliases/UrpdCohort.md)

Defined in: [modules/bitview-client/index.js:1461](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1461)

***

### date

> **date**: `string`

Defined in: [modules/bitview-client/index.js:1460](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1460)

UTC date of the represented block's monotonic timestamp.

***

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:1459](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1459)

Hash of block `height`.

***

### height

> **height**: `number`

Defined in: [modules/bitview-client/index.js:1458](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1458)

Exact published block represented by this distribution.

***

### supply

> **supply**: `number`

Defined in: [modules/bitview-client/index.js:1465](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1465)

Sum of `buckets.supply`, in BTC.

***

### weight

> **weight**: [`UrpdWeight`](../type-aliases/UrpdWeight.md)

Defined in: [modules/bitview-client/index.js:1462](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1462)
