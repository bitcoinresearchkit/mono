[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / SkippedBuilder

# Interface: SkippedBuilder\<T\>

Defined in: [Developer/brk/modules/bitview-client/index.js:1857](https://github.com/bitcoinresearchkit/brk/blob/b971f8d1b413e122481dddc3c9c980b7866ab1d3/modules/bitview-client/index.js#L1857)

## Type Parameters

### T

`T`

## Properties

### fetch

> **fetch**: (`arg?`, `options?`) => `Promise`\<[`SeriesData`](../type-aliases/SeriesData.md)\<`T`\>\>

Defined in: [Developer/brk/modules/bitview-client/index.js:1859](https://github.com/bitcoinresearchkit/brk/blob/b971f8d1b413e122481dddc3c9c980b7866ab1d3/modules/bitview-client/index.js#L1859)

Fetch from skipped position to end

#### Parameters

##### arg?

[`SeriesFetchArg`](../type-aliases/SeriesFetchArg.md)\<`T`\>

##### options?

[`ClientFetchOptions`](ClientFetchOptions.md)\<[`SeriesData`](../type-aliases/SeriesData.md)\<`T`\>\>

#### Returns

`Promise`\<[`SeriesData`](../type-aliases/SeriesData.md)\<`T`\>\>

***

### fetchCsv

> **fetchCsv**: (`options?`) => `Promise`\<`string`\>

Defined in: [Developer/brk/modules/bitview-client/index.js:1860](https://github.com/bitcoinresearchkit/brk/blob/b971f8d1b413e122481dddc3c9c980b7866ab1d3/modules/bitview-client/index.js#L1860)

Fetch as CSV

#### Parameters

##### options?

[`ClientFetchOptions`](ClientFetchOptions.md)\<`string`\>

#### Returns

`Promise`\<`string`\>

***

### take

> **take**: (`n`) => [`RangeBuilder`](RangeBuilder.md)\<`T`\>

Defined in: [Developer/brk/modules/bitview-client/index.js:1858](https://github.com/bitcoinresearchkit/brk/blob/b971f8d1b413e122481dddc3c9c980b7866ab1d3/modules/bitview-client/index.js#L1858)

Take n items after skipped position

#### Parameters

##### n

`number`

#### Returns

[`RangeBuilder`](RangeBuilder.md)\<`T`\>

***

### then

> **then**: [`Thenable`](../type-aliases/Thenable.md)\<`T`\>

Defined in: [Developer/brk/modules/bitview-client/index.js:1861](https://github.com/bitcoinresearchkit/brk/blob/b971f8d1b413e122481dddc3c9c980b7866ab1d3/modules/bitview-client/index.js#L1861)

Thenable
