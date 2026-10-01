[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / UrpdBucket

# Interface: UrpdBucket

Defined in: [Developer/mono/modules/bitview-client/index.js:1394](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1394)

## Properties

### priceFloor

> **priceFloor**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1395](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1395)

Lower bound of the bucket, in USD. Equals the exact realized price for `Raw`.

***

### realizedCap

> **realizedCap**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1397](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1397)

Realized cap contribution in USD: sum of `realized_price * supply` over the coins in this bucket.

***

### supply

> **supply**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1396](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1396)

Supply held with a last-move price inside this bucket, in BTC.

***

### unrealizedPnl

> **unrealizedPnl**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1398](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1398)

Unrealized P&L in USD against the close on the snapshot date: `close * supply - realized_cap`. Can be negative.
