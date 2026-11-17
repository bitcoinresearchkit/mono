/**
 * @typedef {Object} BitviewClientOptions
 * @property {string} baseUrl
 * @property {number} [timeout] - Request timeout in milliseconds, the download included. Default: 30000
 * @property {string|boolean} [browserCache] - Enable browser Cache API with default name (true), custom name (string), or disable (false). No effect in Node.js. Default: true
 * @property {number|boolean} [memCache] - LRU capacity of the in-memory parsed-response cache, which lets 304 responses skip the JSON parse: true/undefined → 1000, false/0 → disabled. Callers of a URL with the same body reader and JSON key-preservation schema share the value it holds, frozen deeply (bytes, a Uint8Array, can't be frozen: don't change them): copy it before changing it, or disable the cache to own every value
 */

/**
 * A JSON value as the memory cache shares it, frozen deeply (`_deepFreeze`): readonly all the way
 * down, `any` and `unknown` left as they are. Plain data only (a series, with its methods, types its
 * own data so).
 * @template T
 * @typedef {unknown extends T ? T : FrozenObject<T>} Frozen
 */

/**
 * `Frozen`'s readonly mapping of a known type (a primitive maps to itself).
 * @template T
 * @typedef {{ readonly [K in keyof T]: Frozen<T[K]> }} FrozenObject
 */

/**
 * @template T
 * @typedef {Object} SeriesDataBase
 * @property {number} version
 * @property {Index} index
 * @property {string} type - Value type (e.g. "Ratio", "Sats", "Cents")
 * @property {number} start - Start index (inclusive)
 * @property {number} end - End index (exclusive)
 * @property {readonly Frozen<T>[]} data - The series data (`null` where a value is missing or undefined), frozen when the memory cache shares it
 * @property {boolean} isDateBased
 * @property {() => number[]} indexes
 * @property {() => number[]} keys - Alias for indexes
 * @property {() => Array<[number, Frozen<T>]>} entries - Get [index, value] pairs
 * @property {() => Map<number, Frozen<T>>} toMap - Convert to Map<index, value>
 */

/** @template T @typedef {SeriesDataBase<T> & Iterable<[number, Frozen<T>]>} SeriesData */

/**
 * @template T
 * @typedef {Object} DateSeriesDataExtras
 * @property {() => globalThis.Date[]} dates - Get dates for each data point
 * @property {() => Array<[globalThis.Date, Frozen<T>]>} dateEntries
 * @property {() => Map<globalThis.Date, Frozen<T>>} toDateMap
 */

/** @template T @typedef {SeriesData<T> & DateSeriesDataExtras<T>} DateSeriesData */
/** @typedef {SeriesData<any>} AnySeriesData */

/** @template T @typedef {PromiseLike<SeriesData<T>>['then']} Thenable */
/** @template T @typedef {PromiseLike<DateSeriesData<T>>['then']} DateThenable */

/**
 * @template T
 * @typedef {Object} SeriesEndpoint
 * @property {(index: number) => RangeBuilder<T>} get
 * @property {(start?: number, end?: number) => RangeBuilder<T>} slice
 * @property {(n: number) => RangeBuilder<T>} first
 * @property {(n: number) => RangeBuilder<T>} last
 * @property {(n: number) => SkippedBuilder<T>} skip - Skip first n items, chain with take()
 * @property {(arg?: SeriesFetchArg<T>, options?: ClientFetchOptions<SeriesData<T>>) => Promise<SeriesData<T>>} fetch - Fetch all data
 * @property {(options?: ClientFetchOptions<string>) => Promise<string>} fetchCsv - Fetch all data as CSV
 * @property {() => Promise<number>} len - Get total number of data points
 * @property {() => Promise<Version>} version
 * @property {Thenable<T>} then - Thenable (await endpoint)
 * @property {string} path
 */

/**
 * @template T
 * @typedef {Object} DateSeriesEndpoint
 * @property {(index: number | globalThis.Date) => DateRangeBuilder<T>} get
 * @property {(start?: number | globalThis.Date, end?: number | globalThis.Date) => DateRangeBuilder<T>} slice
 * @property {(n: number) => DateRangeBuilder<T>} first
 * @property {(n: number) => DateRangeBuilder<T>} last
 * @property {(n: number) => DateSkippedBuilder<T>} skip - Skip first n items, chain with take()
 * @property {(arg?: DateSeriesFetchArg<T>, options?: ClientFetchOptions<DateSeriesData<T>>) => Promise<DateSeriesData<T>>} fetch - Fetch all data
 * @property {(options?: ClientFetchOptions<string>) => Promise<string>} fetchCsv - Fetch all data as CSV
 * @property {() => Promise<number>} len - Get total number of data points
 * @property {() => Promise<Version>} version
 * @property {DateThenable<T>} then - Thenable (await endpoint)
 * @property {string} path
 */

/** @typedef {SeriesEndpoint<any>} AnySeriesEndpoint */

/**
 * @template T
 * @typedef {Object} ClientFetchOptions
 * @property {AbortSignal} [signal] - Abort this request
 * @property {boolean} [cache] - Use HTTP/browser/client caches. Set false for a no-store network fetch.
 * @property {boolean} [memCache] - Use the parsed in-memory response cache, whose values are shared and frozen deeply (bytes can't be frozen: don't change them). Set false for large one-shot reads, or a value of your own to change.
 * @property {(value: T) => void} [onValue] - Receive stale/fresh values as they arrive
 */

/** @template T @typedef {ClientFetchOptions<SeriesData<T>> | ((value: SeriesData<T>) => void)} SeriesFetchArg */
/** @template T @typedef {ClientFetchOptions<DateSeriesData<T>> | ((value: DateSeriesData<T>) => void)} DateSeriesFetchArg */

/** @template T @typedef {Object} SkippedBuilder
 * @property {(n: number) => RangeBuilder<T>} take - Take n items after skipped position
 * @property {(arg?: SeriesFetchArg<T>, options?: ClientFetchOptions<SeriesData<T>>) => Promise<SeriesData<T>>} fetch - Fetch from skipped position to end
 * @property {(options?: ClientFetchOptions<string>) => Promise<string>} fetchCsv
 * @property {Thenable<T>} then
 */

/** @template T @typedef {Object} DateSkippedBuilder
 * @property {(n: number) => DateRangeBuilder<T>} take - Take n items after skipped position
 * @property {(arg?: DateSeriesFetchArg<T>, options?: ClientFetchOptions<DateSeriesData<T>>) => Promise<DateSeriesData<T>>} fetch - Fetch from skipped position to end
 * @property {(options?: ClientFetchOptions<string>) => Promise<string>} fetchCsv
 * @property {DateThenable<T>} then
 */

/** @template T @typedef {Object} RangeBuilder
 * @property {(arg?: SeriesFetchArg<T>, options?: ClientFetchOptions<SeriesData<T>>) => Promise<SeriesData<T>>} fetch - Fetch the selection (one item or a range)
 * @property {(options?: ClientFetchOptions<string>) => Promise<string>} fetchCsv
 * @property {Thenable<T>} then
 */

/** @template T @typedef {Object} DateRangeBuilder
 * @property {(arg?: DateSeriesFetchArg<T>, options?: ClientFetchOptions<DateSeriesData<T>>) => Promise<DateSeriesData<T>>} fetch - Fetch the selection (one item or a range)
 * @property {(options?: ClientFetchOptions<string>) => Promise<string>} fetchCsv
 * @property {DateThenable<T>} then
 */

/**
 * @template T
 * @typedef {Object} SeriesPattern
 * @property {string} name
 * @property {Readonly<Partial<Record<Index, SeriesEndpoint<T>>>>} by - Index endpoints as lazy getters
 * @property {() => readonly Index[]} indexes
 * @property {(index: Index) => SeriesEndpoint<T>|undefined} get
 */

/** @typedef {SeriesPattern<any>} AnySeriesPattern */

/** @template T @typedef {{ etag: string | null, value: T }} _MemEntry */
