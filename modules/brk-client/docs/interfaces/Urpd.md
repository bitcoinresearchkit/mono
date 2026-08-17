[**brk-client**](../README.md)

***

[brk-client](../globals.md) / Urpd

# Interface: Urpd

Defined in: [Developer/brk/modules/brk-client/index.js:1458](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L1458)

## Properties

### aggregation

> **aggregation**: [`UrpdAggregation`](../type-aliases/UrpdAggregation.md)

Defined in: [Developer/brk/modules/brk-client/index.js:1462](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L1462)

Aggregation strategy applied to the buckets.

***

### buckets

> **buckets**: [`UrpdBucket`](UrpdBucket.md)[]

Defined in: [Developer/brk/modules/brk-client/index.js:1465](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L1465)

***

### close

> **close**: `number`

Defined in: [Developer/brk/modules/brk-client/index.js:1463](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L1463)

Close price on `date`, in USD. Anchor for `unrealized_pnl`.

***

### cohort

> **cohort**: [`Cohort`](../type-aliases/Cohort.md)

Defined in: [Developer/brk/modules/brk-client/index.js:1459](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L1459)

***

### date

> **date**: `string`

Defined in: [Developer/brk/modules/brk-client/index.js:1460](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L1460)

***

### totalSupply

> **totalSupply**: `number`

Defined in: [Developer/brk/modules/brk-client/index.js:1464](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L1464)

Sum of `supply` across all buckets, in BTC.

***

### weight

> **weight**: [`UrpdWeight`](../type-aliases/UrpdWeight.md)

Defined in: [Developer/brk/modules/brk-client/index.js:1461](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L1461)

Weighting applied to the source supply.
