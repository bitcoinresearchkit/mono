[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / Urpd

# Interface: Urpd

Defined in: [modules/bitview-client/index.js:1463](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1463)

## Properties

### aggregation

> **aggregation**: [`UrpdAggregation`](../type-aliases/UrpdAggregation.md)

Defined in: [modules/bitview-client/index.js:1469](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1469)

***

### buckets

> **buckets**: [`UrpdBuckets`](UrpdBuckets.md)

Defined in: [modules/bitview-client/index.js:1472](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1472)

***

### close

> **close**: `number`

Defined in: [modules/bitview-client/index.js:1470](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1470)

Price at `height`, in USD. Anchor for `unrealized_pnl`.

***

### cohort

> **cohort**: [`UrpdCohort`](../type-aliases/UrpdCohort.md)

Defined in: [modules/bitview-client/index.js:1467](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1467)

***

### date

> **date**: `string`

Defined in: [modules/bitview-client/index.js:1466](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1466)

UTC date of the represented block's monotonic timestamp.

***

### hash

> **hash**: `string`

Defined in: [modules/bitview-client/index.js:1465](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1465)

Hash of block `height`.

***

### height

> **height**: `number`

Defined in: [modules/bitview-client/index.js:1464](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1464)

Exact published block represented by this distribution.

***

### supply

> **supply**: `number`

Defined in: [modules/bitview-client/index.js:1471](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1471)

Sum of `buckets.supply`, in BTC.

***

### weight

> **weight**: [`UrpdWeight`](../type-aliases/UrpdWeight.md)

Defined in: [modules/bitview-client/index.js:1468](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1468)
