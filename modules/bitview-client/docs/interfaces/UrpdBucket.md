[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / UrpdBucket

# Interface: UrpdBucket

Defined in: [Developer/mono/modules/bitview-client/index.js:1528](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1528)

## Properties

### priceFloor

> **priceFloor**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1529](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1529)

Lower bound of the bucket, in USD. Equals the exact realized price for `Raw`.

***

### realizedCap

> **realizedCap**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1531](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1531)

Realized cap contribution in USD: sum of `realized_price * supply` over the coins in this bucket.

***

### supply

> **supply**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1530](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1530)

Supply held with a last-move price inside this bucket, in BTC.

***

### unrealizedPnl

> **unrealizedPnl**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1532](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1532)

Unrealized P&L in USD against the close on the snapshot date: `close * supply - realized_cap`. Can be negative.
