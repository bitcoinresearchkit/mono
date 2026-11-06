[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / DetailedSeriesCount

# Interface: DetailedSeriesCount

Defined in: [modules/bitview-client/index.js:439](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L439)

## Properties

### byDb

> **byDb**: `object`

Defined in: [modules/bitview-client/index.js:444](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L444)

#### Index Signature

\[`key`: `string`\]: [`SeriesCount`](SeriesCount.md)

***

### distinct

> **distinct**: `number`

Defined in: [modules/bitview-client/index.js:440](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L440)

Number of unique series available (e.g., realized_price, market_cap)

***

### lazy

> **lazy**: `number`

Defined in: [modules/bitview-client/index.js:442](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L442)

Number of lazy (computed on-the-fly) series-index combinations

***

### stored

> **stored**: `number`

Defined in: [modules/bitview-client/index.js:443](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L443)

Number of eager (stored on disk) series-index combinations

***

### total

> **total**: `number`

Defined in: [modules/bitview-client/index.js:441](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L441)

Total number of series-index combinations across all timeframes
