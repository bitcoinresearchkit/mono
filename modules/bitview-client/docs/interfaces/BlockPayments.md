[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BlockPayments

# Interface: BlockPayments

Defined in: [modules/bitview-client/index.js:275](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L275)

## Properties

### counts

> **counts**: `number`[]

Defined in: [modules/bitview-client/index.js:279](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L279)

The readings in each bucket from `first` on, over the oracle's search window for the block.

***

### first

> **first**: `number`

Defined in: [modules/bitview-client/index.js:277](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L277)

The first bucket: bucket `i` holds the readings whose $100 bin is in `[i, i + 1) * step`,
the price `10^(10 - i * step / 200)` dollars at its edge.

***

### height

> **height**: `number`

Defined in: [modules/bitview-client/index.js:276](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L276)
