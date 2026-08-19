[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SeriesPattern2

# Type Alias: SeriesPattern2\<T\>

> **SeriesPattern2**\<`T`\> = `object`

Defined in: [Developer/brk/modules/bitview-client/index.js:2489](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L2489)

## Type Parameters

### T

`T`

## Type Declaration

### by

> **by**: `object`

#### by.day1

> `readonly` **day1**: [`DateSeriesEndpoint`](../interfaces/DateSeriesEndpoint.md)\<`T`\>

#### by.day3

> `readonly` **day3**: [`DateSeriesEndpoint`](../interfaces/DateSeriesEndpoint.md)\<`T`\>

#### by.epoch

> `readonly` **epoch**: [`SeriesEndpoint`](../interfaces/SeriesEndpoint.md)\<`T`\>

#### by.halving

> `readonly` **halving**: [`SeriesEndpoint`](../interfaces/SeriesEndpoint.md)\<`T`\>

#### by.hour1

> `readonly` **hour1**: [`DateSeriesEndpoint`](../interfaces/DateSeriesEndpoint.md)\<`T`\>

#### by.hour12

> `readonly` **hour12**: [`DateSeriesEndpoint`](../interfaces/DateSeriesEndpoint.md)\<`T`\>

#### by.hour4

> `readonly` **hour4**: [`DateSeriesEndpoint`](../interfaces/DateSeriesEndpoint.md)\<`T`\>

#### by.minute10

> `readonly` **minute10**: [`DateSeriesEndpoint`](../interfaces/DateSeriesEndpoint.md)\<`T`\>

#### by.minute30

> `readonly` **minute30**: [`DateSeriesEndpoint`](../interfaces/DateSeriesEndpoint.md)\<`T`\>

#### by.month1

> `readonly` **month1**: [`DateSeriesEndpoint`](../interfaces/DateSeriesEndpoint.md)\<`T`\>

#### by.month3

> `readonly` **month3**: [`DateSeriesEndpoint`](../interfaces/DateSeriesEndpoint.md)\<`T`\>

#### by.month6

> `readonly` **month6**: [`DateSeriesEndpoint`](../interfaces/DateSeriesEndpoint.md)\<`T`\>

#### by.week1

> `readonly` **week1**: [`DateSeriesEndpoint`](../interfaces/DateSeriesEndpoint.md)\<`T`\>

#### by.year1

> `readonly` **year1**: [`DateSeriesEndpoint`](../interfaces/DateSeriesEndpoint.md)\<`T`\>

#### by.year10

> `readonly` **year10**: [`DateSeriesEndpoint`](../interfaces/DateSeriesEndpoint.md)\<`T`\>

### get

> **get**: (`index`) => [`SeriesEndpoint`](../interfaces/SeriesEndpoint.md)\<`T`\> \| `undefined`

#### Parameters

##### index

[`Index`](Index.md)

#### Returns

[`SeriesEndpoint`](../interfaces/SeriesEndpoint.md)\<`T`\> \| `undefined`

### indexes

> **indexes**: () => readonly [`Index`](Index.md)[]

#### Returns

readonly [`Index`](Index.md)[]

### name

> **name**: `string`
