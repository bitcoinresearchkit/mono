[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / Urpd

# Interface: Urpd

Defined in: [Developer/mono/modules/bitview-client/index.js:1509](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1509)

## Properties

### aggregation

> **aggregation**: [`UrpdAggregation`](../type-aliases/UrpdAggregation.md)

Defined in: [Developer/mono/modules/bitview-client/index.js:1513](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1513)

Aggregation strategy applied to the buckets.

***

### buckets

> **buckets**: [`UrpdBucket`](UrpdBucket.md)[]

Defined in: [Developer/mono/modules/bitview-client/index.js:1516](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1516)

***

### close

> **close**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1514](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1514)

Close price on `date`, in USD. Anchor for `unrealized_pnl`.

***

### cohort

> **cohort**: [`Cohort`](../type-aliases/Cohort.md)

Defined in: [Developer/mono/modules/bitview-client/index.js:1510](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1510)

***

### date

> **date**: `string`

Defined in: [Developer/mono/modules/bitview-client/index.js:1511](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1511)

***

### totalSupply

> **totalSupply**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1515](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1515)

Sum of `supply` across all buckets, in BTC.

***

### weight

> **weight**: [`UrpdWeight`](../type-aliases/UrpdWeight.md)

Defined in: [Developer/mono/modules/bitview-client/index.js:1512](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1512)

Weighting applied to the source supply.
