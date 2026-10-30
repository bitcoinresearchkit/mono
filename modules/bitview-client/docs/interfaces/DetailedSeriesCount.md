[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / DetailedSeriesCount

# Interface: DetailedSeriesCount

Defined in: modules/bitview-client/index.js:493

## Properties

### byDb

> **byDb**: `object`

Defined in: modules/bitview-client/index.js:498

#### Index Signature

\[`key`: `string`\]: [`SeriesCount`](SeriesCount.md)

***

### distinct

> **distinct**: `number`

Defined in: modules/bitview-client/index.js:494

Number of unique series available (e.g., realized_price, market_cap)

***

### lazy

> **lazy**: `number`

Defined in: modules/bitview-client/index.js:496

Number of lazy (computed on-the-fly) series-index combinations

***

### stored

> **stored**: `number`

Defined in: modules/bitview-client/index.js:497

Number of eager (stored on disk) series-index combinations

***

### total

> **total**: `number`

Defined in: modules/bitview-client/index.js:495

Total number of series-index combinations across all timeframes
