[**brk-client**](../README.md)

***

[brk-client](../globals.md) / UrpdBucket

# Interface: UrpdBucket

Defined in: [Developer/brk/modules/brk-client/index.js:1477](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L1477)

## Properties

### priceFloor

> **priceFloor**: `number`

Defined in: [Developer/brk/modules/brk-client/index.js:1478](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L1478)

Lower bound of the bucket, in USD. Equals the exact realized price for `Raw`.

***

### realizedCap

> **realizedCap**: `number`

Defined in: [Developer/brk/modules/brk-client/index.js:1480](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L1480)

Realized cap contribution in USD: sum of `realized_price * supply` over the coins in this bucket.

***

### supply

> **supply**: `number`

Defined in: [Developer/brk/modules/brk-client/index.js:1479](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L1479)

Supply held with a last-move price inside this bucket, in BTC.

***

### unrealizedPnl

> **unrealizedPnl**: `number`

Defined in: [Developer/brk/modules/brk-client/index.js:1481](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L1481)

Unrealized P&L in USD against the close on the snapshot date: `close * supply - realized_cap`. Can be negative.
