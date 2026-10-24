[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / DetailedSeriesCount

# Interface: DetailedSeriesCount

Defined in: [modules/bitview-client/index.js:528](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L528)

## Properties

### byDb

> **byDb**: `object`

Defined in: [modules/bitview-client/index.js:533](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L533)

#### Index Signature

\[`key`: `string`\]: [`SeriesCount`](SeriesCount.md)

***

### distinct

> **distinct**: `number`

Defined in: [modules/bitview-client/index.js:529](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L529)

Number of unique series available (e.g., realized_price, market_cap)

***

### lazy

> **lazy**: `number`

Defined in: [modules/bitview-client/index.js:531](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L531)

Number of lazy (computed on-the-fly) series-index combinations

***

### stored

> **stored**: `number`

Defined in: [modules/bitview-client/index.js:532](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L532)

Number of eager (stored on disk) series-index combinations

***

### total

> **total**: `number`

Defined in: [modules/bitview-client/index.js:530](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L530)

Total number of series-index combinations across all timeframes
