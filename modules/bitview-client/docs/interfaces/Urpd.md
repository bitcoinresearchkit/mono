[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / Urpd

# Interface: Urpd

Defined in: [modules/bitview-client/index.js:1565](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1565)

## Properties

### aggregation

> **aggregation**: [`UrpdAggregation`](../type-aliases/UrpdAggregation.md)

Defined in: [modules/bitview-client/index.js:1571](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1571)

***

### buckets

> **buckets**: [`UrpdBuckets`](UrpdBuckets.md)

Defined in: [modules/bitview-client/index.js:1574](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1574)

***

### close

> **close**: `number`

Defined in: [modules/bitview-client/index.js:1572](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1572)

Price at `height`, in USD. Anchor for `unrealized_pnl`.

***

### cohort

> **cohort**: [`UrpdCohort`](../type-aliases/UrpdCohort.md)

Defined in: [modules/bitview-client/index.js:1569](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1569)

***

### date

> **date**: `string`

Defined in: [modules/bitview-client/index.js:1568](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1568)

UTC date of the represented block's monotonic timestamp.

***

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:1567](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1567)

Hash of block `height`.

***

### height

> **height**: `number`

Defined in: [modules/bitview-client/index.js:1566](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1566)

Exact published block represented by this distribution.

***

### supply

> **supply**: `number`

Defined in: [modules/bitview-client/index.js:1573](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1573)

Sum of `buckets.supply`, in BTC.

***

### weight

> **weight**: [`UrpdWeight`](../type-aliases/UrpdWeight.md)

Defined in: [modules/bitview-client/index.js:1570](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1570)
