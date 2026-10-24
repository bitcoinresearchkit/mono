[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / UrpdBuckets

# Interface: UrpdBuckets

Defined in: [modules/bitview-client/index.js:1450](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1450)

## Properties

### price

> **price**: `number`[]

Defined in: [modules/bitview-client/index.js:1451](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1451)

Each bucket's price, in USD: its lower bound when aggregated, and for `raw` the price its
coins' creation prices round to (four significant digits of the price in cents).

***

### realizedCap

> **realizedCap**: `number`[]

Defined in: [modules/bitview-client/index.js:1455](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1455)

Realized cap contribution in USD: `price * supply` summed over the coins in each bucket,
each coin's creation price taken on the price grid (the `raw` bucket price, within 0.05%
of the exact price), rounded to the cent: a bucket worth under half a cent shows 0. The
buckets therefore need not sum to the cohort's realized cap series.

***

### supply

> **supply**: `number`[]

Defined in: [modules/bitview-client/index.js:1453](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1453)

Supply held with a last-move price inside each bucket, in BTC. With a `wakefulness` or
`mobility` weight, the weighted supply, floored to whole satoshis at each grid price.

***

### unrealizedPnl

> **unrealizedPnl**: `number`[]

Defined in: [modules/bitview-client/index.js:1459](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1459)

Unrealized P&L in USD at the represented block: `close * supply` minus each bucket's grid
capital, rounded to the cent once. Can be negative.
