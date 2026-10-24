[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / DetailedSeriesCount

# Interface: DetailedSeriesCount

Defined in: [modules/bitview-client/index.js:513](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L513)

## Properties

### byDb

> **byDb**: `object`

Defined in: [modules/bitview-client/index.js:518](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L518)

#### Index Signature

\[`key`: `string`\]: [`SeriesCount`](SeriesCount.md)

***

### distinct

> **distinct**: `number`

Defined in: [modules/bitview-client/index.js:514](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L514)

Number of unique series available (e.g., realized_price, market_cap)

***

### lazy

> **lazy**: `number`

Defined in: [modules/bitview-client/index.js:516](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L516)

Number of lazy (computed on-the-fly) series-index combinations

***

### stored

> **stored**: `number`

Defined in: [modules/bitview-client/index.js:517](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L517)

Number of eager (stored on disk) series-index combinations

***

### total

> **total**: `number`

Defined in: [modules/bitview-client/index.js:515](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L515)

Total number of series-index combinations across all timeframes
