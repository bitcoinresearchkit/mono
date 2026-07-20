//! JavaScript base client and pattern factory generation.

use std::fmt::Write;

use crate::{
    ClientConstants, ClientMetadata, CohortConstants, GenericSyntax, IndexSetPattern,
    JavaScriptSyntax, StructuralPattern, camel_case_keys, format_json,
    generate_parameterized_field, to_camel_case,
};

/// Generate the base BrkClient class with HTTP functionality.
pub fn generate_base_client(output: &mut String) {
    writeln!(
        output,
        r#"/**
 * @typedef {{Object}} BrkClientOptions
 * @property {{string}} baseUrl - Base URL for the API
 * @property {{number}} [timeout] - Request timeout in milliseconds
 * @property {{string|boolean}} [browserCache] - Enable browser Cache API with default name (true), custom name (string), or disable (false). No effect in Node.js. Default: true
 * @property {{number|boolean}} [memCache] - In-memory parsed-response cache size (LRU). true/undefined → 1000, false/0 → disabled. Lets 304 responses skip the JSON parse entirely. Default: 1000
 */

const _isBrowser = typeof window !== 'undefined' && 'caches' in window;
const _runIdle = (/** @type {{VoidFunction}} */ fn) => (globalThis.requestIdleCallback ?? setTimeout)(fn);
const _defaultBrowserCacheName = '__BRK_CLIENT__';
const _DEFAULT_MEM_CACHE_SIZE = 1000;

/** @template T @typedef {{{{ etag: string | null, value: T }}}} _MemEntry */
/** @param {{*}} v */
const _addCamelGetters = (v) => {{
  if (Array.isArray(v)) {{ v.forEach(_addCamelGetters); return v; }}
  if (v && typeof v === 'object' && v.constructor === Object) {{
    for (const k in v) {{
      if (k.includes('_')) {{
        const c = k.replace(/_([a-z])/g, (_, l) => l.toUpperCase());
        if (!(c in v)) Object.defineProperty(v, c, {{ get() {{ return this[k]; }} }});
      }}
      _addCamelGetters(v[k]);
    }}
  }}
  return v;
}};

/**
 * @param {{string|boolean|undefined}} option
 * @returns {{Promise<Cache | null>}}
 */
const _openBrowserCache = (option) => {{
  if (!_isBrowser || option === false) return Promise.resolve(null);
  const name = typeof option === 'string' ? option : _defaultBrowserCacheName;
  return caches.open(name).catch(() => null);
}};

/**
 * @param {{string}} url
 * @returns {{URL}}
 */
const _parseBaseUrl = (url) => new URL(url, typeof location === 'undefined' ? undefined : location.href);

/**
 * Custom error class for BRK client errors
 */
class BrkError extends Error {{
  /**
   * @param {{string}} message
   * @param {{number}} [status]
   */
  constructor(message, status) {{
    super(message);
    this.name = 'BrkError';
    this.status = status;
  }}
}}

// Date conversion constants and helpers
const _GENESIS = new Date(2009, 0, 3);  // day1 0, week1 0
const _DAY_ONE = new Date(2009, 0, 9);  // day1 1 (6 day gap after genesis)
const _MS_PER_DAY = 86400000;
const _MS_PER_WEEK = 7 * _MS_PER_DAY;
const _EPOCH_MS = 1230768000000;
const _DATE_INDEXES = new Set([
  'minute10', 'minute30',
  'hour1', 'hour4', 'hour12',
  'day1', 'day3', 'week1',
  'month1', 'month3', 'month6',
  'year1', 'year10',
]);

/** @param {{number}} months @returns {{globalThis.Date}} */
const _addMonths = (months) => new Date(2009, months, 1);

/**
 * Convert an index value to a Date for date-based indexes.
 * @param {{Index}} index - The index type
 * @param {{number}} i - The index value
 * @returns {{globalThis.Date}}
 */
function indexToDate(index, i) {{
  switch (index) {{
    case 'minute10': return new Date(_EPOCH_MS + i * 600000);
    case 'minute30': return new Date(_EPOCH_MS + i * 1800000);
    case 'hour1': return new Date(_EPOCH_MS + i * 3600000);
    case 'hour4': return new Date(_EPOCH_MS + i * 14400000);
    case 'hour12': return new Date(_EPOCH_MS + i * 43200000);
    case 'day1': return i === 0 ? _GENESIS : new Date(_DAY_ONE.getTime() + (i - 1) * _MS_PER_DAY);
    case 'day3': return new Date(_EPOCH_MS - 86400000 + i * 259200000);
    case 'week1': return new Date(_GENESIS.getTime() + i * _MS_PER_WEEK);
    case 'month1': return _addMonths(i);
    case 'month3': return _addMonths(i * 3);
    case 'month6': return _addMonths(i * 6);
    case 'year1': return new Date(2009 + i, 0, 1);
    case 'year10': return new Date(2009 + i * 10, 0, 1);
    default: throw new Error(`${{index}} is not a date-based index`);
  }}
}}

/**
 * Convert a Date to an index value for date-based indexes.
 * Returns the floor index (latest index whose date is <= the given date).
 * @param {{Index}} index - The index type
 * @param {{globalThis.Date}} d - The date to convert
 * @returns {{number}}
 */
function dateToIndex(index, d) {{
  const ms = d.getTime();
  switch (index) {{
    case 'minute10': return Math.floor((ms - _EPOCH_MS) / 600000);
    case 'minute30': return Math.floor((ms - _EPOCH_MS) / 1800000);
    case 'hour1': return Math.floor((ms - _EPOCH_MS) / 3600000);
    case 'hour4': return Math.floor((ms - _EPOCH_MS) / 14400000);
    case 'hour12': return Math.floor((ms - _EPOCH_MS) / 43200000);
    case 'day1': {{
      if (ms < _DAY_ONE.getTime()) return 0;
      return 1 + Math.floor((ms - _DAY_ONE.getTime()) / _MS_PER_DAY);
    }}
    case 'day3': return Math.floor((ms - _EPOCH_MS + 86400000) / 259200000);
    case 'week1': return Math.floor((ms - _GENESIS.getTime()) / _MS_PER_WEEK);
    case 'month1': return (d.getFullYear() - 2009) * 12 + d.getMonth();
    case 'month3': return (d.getFullYear() - 2009) * 4 + Math.floor(d.getMonth() / 3);
    case 'month6': return (d.getFullYear() - 2009) * 2 + Math.floor(d.getMonth() / 6);
    case 'year1': return d.getFullYear() - 2009;
    case 'year10': return Math.floor((d.getFullYear() - 2009) / 10);
    default: throw new Error(`${{index}} is not a date-based index`);
  }}
}}

/**
 * Wrap raw series data with helper methods.
 * @template T
 * @param {{SeriesData<T>}} raw - Raw JSON response
 * @returns {{DateSeriesData<T>}}
 */
function _wrapSeriesData(raw) {{
  const {{ index, start, end, data }} = raw;
  const _dateBased = _DATE_INDEXES.has(index);
  return /** @type {{DateSeriesData<T>}} */ ({{
    ...raw,
    isDateBased: _dateBased,
    indexes() {{
      /** @type {{number[]}} */
      const result = [];
      for (let i = start; i < end; i++) result.push(i);
      return result;
    }},
    keys() {{
      return this.indexes();
    }},
    entries() {{
      /** @type {{Array<[number, T]>}} */
      const result = [];
      for (let i = 0; i < data.length; i++) result.push([start + i, data[i]]);
      return result;
    }},
    toMap() {{
      /** @type {{Map<number, T>}} */
      const map = new Map();
      for (let i = 0; i < data.length; i++) map.set(start + i, data[i]);
      return map;
    }},
    *[Symbol.iterator]() {{
      for (let i = 0; i < data.length; i++) yield /** @type {{[number, T]}} */ ([start + i, data[i]]);
    }},
    // DateSeriesData methods (only meaningful for date-based indexes)
    dates() {{
      /** @type {{globalThis.Date[]}} */
      const result = [];
      for (let i = start; i < end; i++) result.push(indexToDate(index, i));
      return result;
    }},
    dateEntries() {{
      /** @type {{Array<[globalThis.Date, T]>}} */
      const result = [];
      for (let i = 0; i < data.length; i++) result.push([indexToDate(index, start + i), data[i]]);
      return result;
    }},
    toDateMap() {{
      /** @type {{Map<globalThis.Date, T>}} */
      const map = new Map();
      for (let i = 0; i < data.length; i++) map.set(indexToDate(index, start + i), data[i]);
      return map;
    }},
  }});
}}

/**
 * @template T
 * @typedef {{Object}} SeriesDataBase
 * @property {{number}} version - Version of the series data
 * @property {{Index}} index - The index type used for this query
 * @property {{string}} type - Value type (e.g. "f32", "u64", "Sats")
 * @property {{number}} start - Start index (inclusive)
 * @property {{number}} end - End index (exclusive)
 * @property {{string}} stamp - ISO 8601 timestamp of when the response was generated
 * @property {{T[]}} data - The series data
 * @property {{boolean}} isDateBased - Whether this series uses a date-based index
 * @property {{() => number[]}} indexes - Get index numbers
 * @property {{() => number[]}} keys - Get keys as index numbers (alias for indexes)
 * @property {{() => Array<[number, T]>}} entries - Get [index, value] pairs
 * @property {{() => Map<number, T>}} toMap - Convert to Map<index, value>
 */

/** @template T @typedef {{SeriesDataBase<T> & Iterable<[number, T]>}} SeriesData */

/**
 * @template T
 * @typedef {{Object}} DateSeriesDataExtras
 * @property {{() => globalThis.Date[]}} dates - Get dates for each data point
 * @property {{() => Array<[globalThis.Date, T]>}} dateEntries - Get [date, value] pairs
 * @property {{() => Map<globalThis.Date, T>}} toDateMap - Convert to Map<date, value>
 */

/** @template T @typedef {{SeriesData<T> & DateSeriesDataExtras<T>}} DateSeriesData */
/** @typedef {{SeriesData<any>}} AnySeriesData */

/** @template T @typedef {{(onfulfilled?: (value: SeriesData<T>) => any, onrejected?: (reason: Error) => never) => Promise<SeriesData<T>>}} Thenable */
/** @template T @typedef {{(onfulfilled?: (value: DateSeriesData<T>) => any, onrejected?: (reason: Error) => never) => Promise<DateSeriesData<T>>}} DateThenable */

/**
 * @template T
 * @typedef {{Object}} SeriesEndpoint
 * @property {{(index: number) => SingleItemBuilder<T>}} get - Get single item at index
 * @property {{(start?: number, end?: number) => RangeBuilder<T>}} slice - Slice by index
 * @property {{(n: number) => RangeBuilder<T>}} first - Get first n items
 * @property {{(n: number) => RangeBuilder<T>}} last - Get last n items
 * @property {{(n: number) => SkippedBuilder<T>}} skip - Skip first n items, chain with take()
 * @property {{(arg?: SeriesFetchArg<T>, options?: ClientFetchOptions<SeriesData<T>>) => Promise<SeriesData<T>>}} fetch - Fetch all data
 * @property {{(options?: ClientFetchOptions<string>) => Promise<string>}} fetchCsv - Fetch all data as CSV
 * @property {{() => Promise<number>}} len - Get total number of data points
 * @property {{() => Promise<Version>}} version - Get the current version of the series
 * @property {{Thenable<T>}} then - Thenable (await endpoint)
 * @property {{string}} path - The endpoint path
 */

/**
 * @template T
 * @typedef {{Object}} DateSeriesEndpoint
 * @property {{(index: number | globalThis.Date) => DateSingleItemBuilder<T>}} get - Get single item at index or Date
 * @property {{(start?: number | globalThis.Date, end?: number | globalThis.Date) => DateRangeBuilder<T>}} slice - Slice by index or Date
 * @property {{(n: number) => DateRangeBuilder<T>}} first - Get first n items
 * @property {{(n: number) => DateRangeBuilder<T>}} last - Get last n items
 * @property {{(n: number) => DateSkippedBuilder<T>}} skip - Skip first n items, chain with take()
 * @property {{(arg?: DateSeriesFetchArg<T>, options?: ClientFetchOptions<DateSeriesData<T>>) => Promise<DateSeriesData<T>>}} fetch - Fetch all data
 * @property {{(options?: ClientFetchOptions<string>) => Promise<string>}} fetchCsv - Fetch all data as CSV
 * @property {{() => Promise<number>}} len - Get total number of data points
 * @property {{() => Promise<Version>}} version - Get the current version of the series
 * @property {{DateThenable<T>}} then - Thenable (await endpoint)
 * @property {{string}} path - The endpoint path
 */

/** @typedef {{SeriesEndpoint<any>}} AnySeriesEndpoint */

/**
 * @template T
 * @typedef {{Object}} ClientFetchOptions
 * @property {{AbortSignal}} [signal] - Abort this request
 * @property {{boolean}} [cache] - Use HTTP/browser/client caches. Set false for a no-store network fetch.
 * @property {{boolean}} [memCache] - Use the parsed in-memory response cache. Set false for large one-shot reads.
 * @property {{(value: T) => void}} [onValue] - Receive stale/fresh values as they arrive
 */

/** @template T @typedef {{ClientFetchOptions<SeriesData<T>> | ((value: SeriesData<T>) => void)}} SeriesFetchArg */
/** @template T @typedef {{ClientFetchOptions<DateSeriesData<T>> | ((value: DateSeriesData<T>) => void)}} DateSeriesFetchArg */

/** @template T @typedef {{Object}} SingleItemBuilder
 * @property {{(arg?: SeriesFetchArg<T>, options?: ClientFetchOptions<SeriesData<T>>) => Promise<SeriesData<T>>}} fetch - Fetch the item
 * @property {{(options?: ClientFetchOptions<string>) => Promise<string>}} fetchCsv - Fetch as CSV
 * @property {{Thenable<T>}} then - Thenable
 */

/** @template T @typedef {{Object}} DateSingleItemBuilder
 * @property {{(arg?: DateSeriesFetchArg<T>, options?: ClientFetchOptions<DateSeriesData<T>>) => Promise<DateSeriesData<T>>}} fetch - Fetch the item
 * @property {{(options?: ClientFetchOptions<string>) => Promise<string>}} fetchCsv - Fetch as CSV
 * @property {{DateThenable<T>}} then - Thenable
 */

/** @template T @typedef {{Object}} SkippedBuilder
 * @property {{(n: number) => RangeBuilder<T>}} take - Take n items after skipped position
 * @property {{(arg?: SeriesFetchArg<T>, options?: ClientFetchOptions<SeriesData<T>>) => Promise<SeriesData<T>>}} fetch - Fetch from skipped position to end
 * @property {{(options?: ClientFetchOptions<string>) => Promise<string>}} fetchCsv - Fetch as CSV
 * @property {{Thenable<T>}} then - Thenable
 */

/** @template T @typedef {{Object}} DateSkippedBuilder
 * @property {{(n: number) => DateRangeBuilder<T>}} take - Take n items after skipped position
 * @property {{(arg?: DateSeriesFetchArg<T>, options?: ClientFetchOptions<DateSeriesData<T>>) => Promise<DateSeriesData<T>>}} fetch - Fetch from skipped position to end
 * @property {{(options?: ClientFetchOptions<string>) => Promise<string>}} fetchCsv - Fetch as CSV
 * @property {{DateThenable<T>}} then - Thenable
 */

/** @template T @typedef {{Object}} RangeBuilder
 * @property {{(arg?: SeriesFetchArg<T>, options?: ClientFetchOptions<SeriesData<T>>) => Promise<SeriesData<T>>}} fetch - Fetch the range
 * @property {{(options?: ClientFetchOptions<string>) => Promise<string>}} fetchCsv - Fetch as CSV
 * @property {{Thenable<T>}} then - Thenable
 */

/** @template T @typedef {{Object}} DateRangeBuilder
 * @property {{(arg?: DateSeriesFetchArg<T>, options?: ClientFetchOptions<DateSeriesData<T>>) => Promise<DateSeriesData<T>>}} fetch - Fetch the range
 * @property {{(options?: ClientFetchOptions<string>) => Promise<string>}} fetchCsv - Fetch as CSV
 * @property {{DateThenable<T>}} then - Thenable
 */

/**
 * @template T
 * @typedef {{Object}} SeriesPattern
 * @property {{string}} name - The series name
 * @property {{Readonly<Partial<Record<Index, SeriesEndpoint<T>>>>}} by - Index endpoints as lazy getters
 * @property {{() => readonly Index[]}} indexes - Get the list of available indexes
 * @property {{(index: Index) => SeriesEndpoint<T>|undefined}} get - Get an endpoint for a specific index
 */

/** @typedef {{SeriesPattern<any>}} AnySeriesPattern */

/**
 * Create a series endpoint builder with typestate pattern.
 * @template T
 * @param {{BrkClient}} client
 * @param {{string}} name - The series vec name
 * @param {{Index}} index - The index name
 * @returns {{DateSeriesEndpoint<T>}}
 */
function _endpoint(client, name, index) {{
  const p = `/api/series/${{name}}/${{index}}`;

  /**
   * @param {{number}} [start]
   * @param {{number}} [end]
   * @param {{string}} [format]
   * @returns {{string}}
   */
  const buildPath = (start, end, format) => {{
    const params = new URLSearchParams();
    if (start !== undefined) params.set('start', String(start));
    if (end !== undefined) params.set('end', String(end));
    if (format) params.set('format', format);
    const query = params.toString();
    return query ? `${{p}}?${{query}}` : p;
  }};

  /**
   * @param {{number}} [start]
   * @param {{number}} [end]
   * @returns {{DateRangeBuilder<T>}}
   */
  const rangeBuilder = (start, end) => ({{
    fetch(arg, options) {{ return client._fetchSeriesData(buildPath(start, end), arg, options); }},
    fetchCsv(options) {{ return client.getText(buildPath(start, end, 'csv'), options); }},
    then(resolve, reject) {{ return this.fetch().then(resolve, reject); }},
  }});

  /**
   * @param {{number}} idx
   * @returns {{DateSingleItemBuilder<T>}}
   */
  const singleItemBuilder = (idx) => ({{
    fetch(arg, options) {{ return client._fetchSeriesData(buildPath(idx, idx + 1), arg, options); }},
    fetchCsv(options) {{ return client.getText(buildPath(idx, idx + 1, 'csv'), options); }},
    then(resolve, reject) {{ return this.fetch().then(resolve, reject); }},
  }});

  /**
   * @param {{number}} start
   * @returns {{DateSkippedBuilder<T>}}
   */
  const skippedBuilder = (start) => ({{
    take(n) {{ return rangeBuilder(start, start + n); }},
    fetch(arg, options) {{ return client._fetchSeriesData(buildPath(start, undefined), arg, options); }},
    fetchCsv(options) {{ return client.getText(buildPath(start, undefined, 'csv'), options); }},
    then(resolve, reject) {{ return this.fetch().then(resolve, reject); }},
  }});

  /** @type {{DateSeriesEndpoint<T>}} */
  const endpoint = {{
    get(idx) {{ if (idx instanceof Date) idx = dateToIndex(index, idx); return singleItemBuilder(idx); }},
    slice(start, end) {{
      if (start instanceof Date) start = dateToIndex(index, start);
      if (end instanceof Date) end = dateToIndex(index, end);
      return rangeBuilder(start, end);
    }},
    first(n) {{ return rangeBuilder(undefined, n); }},
    last(n) {{ return n === 0 ? rangeBuilder(undefined, 0) : rangeBuilder(-n, undefined); }},
    skip(n) {{ return skippedBuilder(n); }},
    fetch(arg, options) {{ return client._fetchSeriesData(buildPath(), arg, options); }},
    fetchCsv(options) {{ return client.getText(buildPath(undefined, undefined, 'csv'), options); }},
    len() {{ return client.getSeriesLen(name, index); }},
    version() {{ return client.getSeriesVersion(name, index); }},
    then(resolve, reject) {{ return this.fetch().then(resolve, reject); }},
    get path() {{ return p; }},
  }};

  return endpoint;
}}

/**
 * Base HTTP client for making requests with caching support
 */
class BrkClientBase {{
  /**
   * @param {{BrkClientOptions|string}} options
   */
  constructor(options) {{
    const isString = typeof options === 'string';
    const rawUrl = isString ? options : options.baseUrl;
    this.baseUrl = rawUrl.endsWith('/') ? rawUrl.slice(0, -1) : rawUrl;
    const url = _parseBaseUrl(this.baseUrl);
    this.url = url.href.endsWith('/') ? url.href.slice(0, -1) : url.href;
    this.domain = url.hostname;
    this.timeout = isString ? 5000 : (options.timeout ?? 5000);
    /** @type {{Promise<Cache | null>}} */
    this._browserCachePromise = _openBrowserCache(isString ? undefined : options.browserCache);
    /** @type {{Cache | null}} */
    this._browserCache = null;
    this._browserCachePromise.then(c => this._browserCache = c);
    const memOpt = isString ? undefined : options.memCache;
    this._memCacheMax = memOpt === false || memOpt === 0
      ? 0
      : (typeof memOpt === 'number' ? memOpt : _DEFAULT_MEM_CACHE_SIZE);
    /** @type {{Map<string, _MemEntry<unknown>>}} */
    this._memCache = new Map();
  }}

  /**
   * @template T
   * @param {{string}} key
   * @returns {{_MemEntry<T> | undefined}}
   */
  _memGet(key) {{
    if (!this._memCacheMax) return undefined;
    const hit = this._memCache.get(key);
    if (!hit) return undefined;
    this._memCache.delete(key);
    this._memCache.set(key, hit);
    return /** @type {{_MemEntry<T>}} */ (hit);
  }}

  /**
   * @param {{string}} key
   * @param {{string | null}} etag
   * @param {{unknown}} value
   */
  _memSet(key, etag, value) {{
    if (!this._memCacheMax) return;
    if (this._memCache.has(key)) this._memCache.delete(key);
    else if (this._memCache.size >= this._memCacheMax) {{
      const oldest = this._memCache.keys().next().value;
      if (oldest !== undefined) this._memCache.delete(oldest);
    }}
    this._memCache.set(key, {{ etag, value }});
  }}

  /**
   * @param {{string}} path
   * @param {{{{ signal?: AbortSignal, cache?: boolean }}}} [options]
   * @returns {{Promise<Response>}}
   */
  async get(path, {{ signal, cache = true }} = {{}}) {{
    const url = `${{this.baseUrl}}${{path}}`;
    const signals = [AbortSignal.timeout(this.timeout)];
    if (signal) signals.push(signal);
    /** @type {{RequestInit}} */
    const init = {{ signal: AbortSignal.any(signals) }};
    if (!cache) init.cache = 'no-store';
    const res = await fetch(url, init);
    if (!res.ok) throw new BrkError(`HTTP ${{res.status}}: ${{url}}`, res.status);
    return res;
  }}

  /**
   * Make a GET request with layered caching.
   *
   * Contract:
   * - The returned Promise resolves with the **freshest** value (post-revalidation).
   * - `onValue` fires once with the freshest value, or twice if a stale snapshot
   *   could be shown first (stale-while-revalidate). On a 304 there is no second fire.
   *
   * Layers:
   * - L1 (memCache): in-memory parsed values keyed by URL+ETag. Lets 304s skip the parse entirely.
   * - L2 (browserCache): Cache API, survives reload and feeds onValue fast on cold start.
   *
   * @template T
   * @param {{string}} path
   * @param {{(res: Response) => Promise<T>}} parse - Response body reader
   * @param {{ClientFetchOptions<T>}} [options]
   * @returns {{Promise<T>}}
   */
  async _getCached(path, parse, {{ onValue, signal, cache = true, memCache = true }} = {{}}) {{
    if (!cache) {{
      const res = await this.get(path, {{ signal, cache }});
      const value = await parse(res);
      if (onValue) onValue(value);
      return value;
    }}

    const url = `${{this.baseUrl}}${{path}}`;
    const useMemCache = memCache !== false;
    /** @type {{_MemEntry<T> | undefined}} */
    const memHit = useMemCache ? this._memGet(url) : undefined;
    const browserCache = this._browserCache;

    // L1 fast path: deliver from memCache, revalidate via network.
    // ETag match → zero parse, zero clone, zero cache write, no second onValue fire.
    if (memHit) {{
      if (onValue) onValue(memHit.value);
      try {{
        const res = await this.get(path, {{ signal }});
        const netEtag = res.headers.get('ETag');
        if (netEtag && netEtag === memHit.etag) return memHit.value;
        const cloned = browserCache ? res.clone() : null;
        const value = await parse(res);
        if (useMemCache) this._memSet(url, netEtag, value);
        if (onValue) onValue(value);
        if (cloned && browserCache) {{
          const cacheStore = browserCache;
          _runIdle(() => cacheStore.put(url, cloned));
        }}
        return value;
      }} catch {{
        return memHit.value;
      }}
    }}

    // L1 miss: race browserCache (stale snapshot) vs network (fresh).
    let networkSettled = false;
    const stalePromise = onValue && browserCache
      ? browserCache.match(url).then(async (res) => {{
          if (!res || networkSettled) return null;
          const value = await parse(res);
          if (networkSettled) return value;
          if (useMemCache) this._memSet(url, res.headers.get('ETag'), value);
          onValue(value);
          return value;
        }}).catch(() => null)
      : null;

    try {{
      const res = await this.get(path, {{ signal }});
      networkSettled = true;
      const netEtag = res.headers.get('ETag');
      // Stale won and populated memCache with matching ETag → reuse, skip parse + second onValue.
      const populated = useMemCache ? /** @type {{_MemEntry<T> | undefined}} */ (this._memGet(url)) : undefined;
      if (populated && netEtag && netEtag === populated.etag) return populated.value;
      const cloned = browserCache ? res.clone() : null;
      const value = await parse(res);
      if (useMemCache) this._memSet(url, netEtag, value);
      if (onValue) onValue(value);
      if (cloned && browserCache) {{
        const cacheStore = browserCache;
        _runIdle(() => cacheStore.put(url, cloned));
      }}
      return value;
    }} catch (e) {{
      const stale = await stalePromise;
      if (stale != null) return stale;
      throw e;
    }}
  }}

  /**
   * Make a GET request expecting a JSON response. Cached and supports `onValue`.
   * @template T
   * @param {{string}} path
   * @param {{ClientFetchOptions<T>}} [options]
   * @returns {{Promise<T>}}
   */
  getJson(path, options) {{
    return this._getCached(path, async (res) => _addCamelGetters(await res.json()), options);
  }}

  /**
   * Make a GET request expecting a text response (text/plain, text/csv, ...).
   * Cached and supports `onValue`, same as `getJson`.
   * @param {{string}} path
   * @param {{ClientFetchOptions<string>}} [options]
   * @returns {{Promise<string>}}
   */
  getText(path, options) {{
    return this._getCached(path, (res) => res.text(), options);
  }}

  /**
   * Make a GET request expecting binary data (application/octet-stream).
   * Cached and supports `onValue`, same as `getJson`.
   * @param {{string}} path
   * @param {{ClientFetchOptions<Uint8Array>}} [options]
   * @returns {{Promise<Uint8Array>}}
   */
  getBytes(path, options) {{
    return this._getCached(path, async (res) => new Uint8Array(await res.arrayBuffer()), options);
  }}

  /**
   * Make a POST request with a string body.
   *
   * POST responses are uncached and never invoke `onValue` — every call hits
   * the network with the same body and returns the upstream response.
   *
   * @param {{string}} path
   * @param {{string}} body
   * @param {{{{ signal?: AbortSignal }}}} [options]
   * @returns {{Promise<Response>}}
   */
  async post(path, body, {{ signal }} = {{}}) {{
    const url = `${{this.baseUrl}}${{path}}`;
    const signals = [AbortSignal.timeout(this.timeout)];
    if (signal) signals.push(signal);
    const res = await fetch(url, {{
      method: 'POST',
      body,
      signal: AbortSignal.any(signals),
    }});
    if (!res.ok) throw new BrkError(`HTTP ${{res.status}}: ${{url}}`, res.status);
    return res;
  }}

  /**
   * Make a POST request expecting a JSON response.
   * @template T
   * @param {{string}} path
   * @param {{string}} body
   * @param {{{{ signal?: AbortSignal }}}} [options]
   * @returns {{Promise<T>}}
   */
  async postJson(path, body, options) {{
    const res = await this.post(path, body, options);
    return _addCamelGetters(await res.json());
  }}

  /**
   * Make a POST request expecting a text response.
   * @param {{string}} path
   * @param {{string}} body
   * @param {{{{ signal?: AbortSignal }}}} [options]
   * @returns {{Promise<string>}}
   */
  async postText(path, body, options) {{
    const res = await this.post(path, body, options);
    return res.text();
  }}

  /**
   * Make a POST request expecting binary data (application/octet-stream).
   * @param {{string}} path
   * @param {{string}} body
   * @param {{{{ signal?: AbortSignal }}}} [options]
   * @returns {{Promise<Uint8Array>}}
   */
  async postBytes(path, body, options) {{
    const res = await this.post(path, body, options);
    return new Uint8Array(await res.arrayBuffer());
  }}

  /**
   * Fetch series data and wrap with helper methods (internal)
   * @template T
   * @param {{string}} path
   * @param {{DateSeriesFetchArg<T>}} [arg]
   * @param {{ClientFetchOptions<DateSeriesData<T>>}} [options]
   * @returns {{Promise<DateSeriesData<T>>}}
   */
  async _fetchSeriesData(path, arg, options) {{
    const requestOptions = typeof arg === 'function'
      ? {{ ...(options ?? {{}}), onValue: arg }}
      : {{ ...(arg ?? {{}}), ...(options ?? {{}}) }};
    const onValue = requestOptions.onValue;
    const wrappedOnValue = onValue ? (/** @type {{SeriesData<T>}} */ raw) => onValue(_wrapSeriesData(raw)) : undefined;
    const raw = await this.getJson(path, {{ ...requestOptions, onValue: wrappedOnValue }});
    return _wrapSeriesData(raw);
  }}
}}

/**
 * Build series name with suffix.
 * @param {{string}} acc - Accumulated prefix
 * @param {{string}} s - Series suffix
 * @returns {{string}}
 */
const _m = (acc, s) => s ? (acc ? `${{acc}}_${{s}}` : s) : acc;

/**
 * Build series name with prefix.
 * @param {{string}} prefix - Prefix to prepend
 * @param {{string}} acc - Accumulated name
 * @returns {{string}}
 */
const _p = (prefix, acc) => acc ? `${{prefix}}_${{acc}}` : prefix;

"#
    )
    .unwrap();
    output.push_str(r##"
const _MASK_64 = 0xffffffffffffffffn;
const _RAPIDHASH_SECRETS = /** @type {const} */ ([
  0x2d358dccaa6c78a5n,
  0x8bb84b93962eacc9n,
  0x4b33a62ed433d4a3n,
  0x4d5a2da51de1aa47n,
  0xa0761d6478bd642fn,
  0xe7037ed1a0b428dbn,
  0x90ed1765281c388cn,
]);
const _RAPIDHASH_SEED = _rapidHashSeed(0n);

/** @param {bigint} value */
function _u64(value) {
  return value & _MASK_64;
}

/** @param {bigint} left @param {bigint} right */
function _rapidMix(left, right) {
  const result = _u64(left) * _u64(right);
  return _u64(result) ^ _u64(result >> 64n);
}

/** @param {bigint} left @param {bigint} right @returns {[bigint, bigint]} */
function _rapidMum(left, right) {
  const result = _u64(left) * _u64(right);
  return [_u64(result), _u64(result >> 64n)];
}

/** @param {bigint} seed */
function _rapidHashSeed(seed) {
  return _u64(seed ^ _rapidMix(seed ^ _RAPIDHASH_SECRETS[2], _RAPIDHASH_SECRETS[1]));
}

/** @param {Uint8Array} bytes @param {number} offset */
function _readU32(bytes, offset) {
  return (
    BigInt(bytes[offset]) |
    (BigInt(bytes[offset + 1]) << 8n) |
    (BigInt(bytes[offset + 2]) << 16n) |
    (BigInt(bytes[offset + 3]) << 24n)
  );
}

/** @param {Uint8Array} bytes @param {number} offset */
function _readU64(bytes, offset) {
  return _readU32(bytes, offset) | (_readU32(bytes, offset + 4) << 32n);
}

/** @param {Uint8Array | ArrayBuffer | ArrayBufferView | number[]} payload */
function _asUint8Array(payload) {
  if (payload instanceof Uint8Array) return payload;
  if (payload instanceof ArrayBuffer) return new Uint8Array(payload);
  if (ArrayBuffer.isView(payload)) return new Uint8Array(payload.buffer, payload.byteOffset, payload.byteLength);
  if (Array.isArray(payload)) return new Uint8Array(payload);
  throw new Error("Expected address payload bytes");
}

/** @param {Uint8Array | ArrayBuffer | ArrayBufferView | number[]} payload */
function _rapidHashV3(payload) {
  const bytes = _asUint8Array(payload);
  const length = bytes.length;
  if (length === 0) throw new Error("Expected a non-empty address payload");
  if (length > 65) throw new Error("Expected at most 65 address payload bytes");

  let seed = _RAPIDHASH_SEED;
  let a = 0n;
  let b = 0n;
  let remainder;

  if (length <= 16) {
    if (length >= 4) {
      seed ^= BigInt(length);
      if (length >= 8) {
        a ^= _readU64(bytes, 0);
        b ^= _readU64(bytes, length - 8);
      } else {
        a ^= _readU32(bytes, 0);
        b ^= _readU32(bytes, length - 4);
      }
    } else if (length > 0) {
      a ^= (BigInt(bytes[0]) << 45n) | BigInt(bytes[length - 1]);
      b ^= BigInt(bytes[length >> 1]);
    }
    remainder = BigInt(length);
  } else {
    seed = _rapidMix(_readU64(bytes, 0) ^ _RAPIDHASH_SECRETS[2], _readU64(bytes, 8) ^ seed);
    if (length > 32) {
      seed = _rapidMix(_readU64(bytes, 16) ^ _RAPIDHASH_SECRETS[2], _readU64(bytes, 24) ^ seed);
      if (length > 48) {
        seed = _rapidMix(_readU64(bytes, 32) ^ _RAPIDHASH_SECRETS[1], _readU64(bytes, 40) ^ seed);
        if (length > 64) {
          seed = _rapidMix(_readU64(bytes, 48) ^ _RAPIDHASH_SECRETS[1], _readU64(bytes, 56) ^ seed);
        }
      }
    }
    remainder = BigInt(length);
    a ^= _readU64(bytes, length - 16) ^ remainder;
    b ^= _readU64(bytes, length - 8);
  }

  a ^= _RAPIDHASH_SECRETS[1];
  b ^= seed;
  [a, b] = _rapidMum(a, b);
  return _rapidMix(a ^ 0xaaaaaaaaaaaaaaaan, b ^ _RAPIDHASH_SECRETS[1] ^ remainder);
}

/** @param {number} nibbles */
function _validateHashPrefixNibbles(nibbles) {
  if (!Number.isInteger(nibbles) || nibbles < 1 || nibbles > 16) {
    throw new Error("Expected hash-prefix length from 1 to 16 hex nibbles");
  }
}

/** @param {OutputType} addrType @returns {number[]} */
function _addressPayloadLengths(addrType) {
  switch (addrType) {
    case "p2a": return [2];
    case "p2pk33": return [33];
    case "p2pk65": return [65];
    case "p2pkh":
    case "p2sh":
    case "p2wpkh": return [20];
    case "p2wsh":
    case "p2tr": return [32];
    default:
      throw new Error(`Unsupported address type for address payload hash-prefix: ${addrType}`);
  }
}

/**
 * @param {OutputType} addrType
 * @param {Uint8Array | ArrayBuffer | ArrayBufferView | number[]} payload
 */
function _validateAddressPayloadForType(addrType, payload) {
  const length = _asUint8Array(payload).length;
  const expected = _addressPayloadLengths(addrType);
  if (!expected.includes(length)) {
    throw new Error(`Expected ${addrType} address payload length ${expected.join(" or ")} bytes`);
  }
}

/**
 * Compute the RapidHash v3 hash-prefix used by `/api/address/hash-prefix/{addr_type}/{prefix}`.
 * @param {Uint8Array | ArrayBuffer | ArrayBufferView | number[]} payload - Raw address payload bytes
 * @param {number} nibbles - Prefix length from 1 to 16 hex nibbles
 * @returns {string}
 */
function addressPayloadHashPrefix(payload, nibbles) {
  _validateHashPrefixNibbles(nibbles);
  return _rapidHashV3(payload).toString(16).padStart(16, "0").slice(0, nibbles);
}

"##);
}

/// Generate static constants for the BrkClient class.
pub fn generate_static_constants(output: &mut String) {
    let constants = ClientConstants::collect();

    // VERSION, INDEXES, POOL_ID_TO_POOL_NAME
    writeln!(output, "  VERSION = \"{}\";\n", constants.version).unwrap();
    write_static_const(output, "INDEXES", &format_json(&constants.indexes));
    write_static_const(
        output,
        "POOL_ID_TO_POOL_NAME",
        &format_json(&constants.pool_map),
    );

    // Cohort constants with camelCase keys
    for (name, value) in CohortConstants::all() {
        write_static_const(output, name, &format_json(&camel_case_keys(value)));
    }

    // Helper methods
    writeln!(
        output,
        r#"  /**
   * Convert an index value to a Date for date-based indexes.
   * @param {{Index}} index - The index type
   * @param {{number}} i - The index value
   * @returns {{globalThis.Date}}
   */
  indexToDate(index, i) {{
    return indexToDate(index, i);
  }}

  /**
   * Convert a Date to an index value for date-based indexes.
   * @param {{Index}} index - The index type
   * @param {{globalThis.Date}} d - The date to convert
   * @returns {{number}}
   */
  dateToIndex(index, d) {{
    return dateToIndex(index, d);
  }}

"#
    )
    .unwrap();
}

fn indent_json_const(json: &str) -> String {
    json.lines()
        .enumerate()
        .map(|(i, line)| {
            if i == 0 {
                line.to_string()
            } else {
                format!("  {}", line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn write_static_const(output: &mut String, name: &str, json: &str) {
    writeln!(
        output,
        "  {} = /** @type {{const}} */ ({});\n",
        name,
        indent_json_const(json)
    )
    .unwrap();
}

/// Generate index accessor factory functions.
pub fn generate_index_accessors(output: &mut String, patterns: &[IndexSetPattern]) {
    if patterns.is_empty() {
        return;
    }

    writeln!(output, "// Index group constants and factory\n").unwrap();

    // Generate index array constants (e.g., _i1 = ["day1", "height"])
    for (i, pattern) in patterns.iter().enumerate() {
        write!(output, "const _i{} = /** @type {{const}} */ ([", i + 1).unwrap();
        for (j, index) in pattern.indexes.iter().enumerate() {
            if j > 0 {
                write!(output, ", ").unwrap();
            }
            write!(output, "\"{}\"", index.name()).unwrap();
        }
        writeln!(output, "]);").unwrap();
    }
    writeln!(output).unwrap();

    // Generate ONE generic series pattern factory
    writeln!(
        output,
        r#"/**
 * Generic series pattern factory.
 * @template T
 * @param {{BrkClient}} client
 * @param {{string}} name - The series vec name
 * @param {{readonly Index[]}} indexes - The supported indexes
 */
function _mp(client, name, indexes) {{
  const by = {{}};
  for (const idx of indexes) {{
    Object.defineProperty(by, idx, {{
      get() {{ return _endpoint(client, name, idx); }},
      enumerable: true,
      configurable: true
    }});
  }}
  return {{
    name,
    by,
    /** @returns {{readonly Index[]}} */
    indexes() {{ return indexes; }},
    /** @param {{Index}} index @returns {{SeriesEndpoint<T>|undefined}} */
    get(index) {{ return indexes.includes(index) ? _endpoint(client, name, index) : undefined; }}
  }};
}}
"#
    )
    .unwrap();

    // Generate typedefs and thin wrapper functions
    for (i, pattern) in patterns.iter().enumerate() {
        // Generate typedef for type safety
        let by_fields: Vec<String> = pattern
            .indexes
            .iter()
            .map(|idx| {
                let builder = if idx.is_date_based() {
                    "DateSeriesEndpoint"
                } else {
                    "SeriesEndpoint"
                };
                format!("readonly {}: {}<T>", idx.name(), builder)
            })
            .collect();
        let by_type = format!("{{ {} }}", by_fields.join(", "));

        writeln!(
            output,
            "/** @template T @typedef {{{{ name: string, by: {}, indexes: () => readonly Index[], get: (index: Index) => SeriesEndpoint<T>|undefined }}}} {} */",
            by_type, pattern.name
        )
        .unwrap();

        // Generate thin wrapper that calls the generic factory
        writeln!(
            output,
            "/** @template T @param {{BrkClient}} client @param {{string}} name @returns {{{}<T>}} */",
            pattern.name
        )
        .unwrap();
        writeln!(
            output,
            "function create{}(client, name) {{ return /** @type {{{}<T>}} */ (_mp(client, name, _i{})); }}",
            pattern.name,
            pattern.name,
            i + 1
        )
        .unwrap();
    }
    writeln!(output).unwrap();
}

/// Generate structural pattern factory functions.
pub fn generate_structural_patterns(
    output: &mut String,
    patterns: &[StructuralPattern],
    metadata: &ClientMetadata,
) {
    if patterns.is_empty() {
        return;
    }

    writeln!(output, "// Reusable structural pattern factories\n").unwrap();

    for pattern in patterns {
        // Generate typedef
        writeln!(output, "/**").unwrap();
        if pattern.is_generic {
            writeln!(output, " * @template T").unwrap();
        }
        writeln!(output, " * @typedef {{Object}} {}", pattern.name).unwrap();
        for field in &pattern.fields {
            let js_type = metadata.field_type_annotation(
                field,
                pattern.is_generic,
                None,
                GenericSyntax::JAVASCRIPT,
            );
            writeln!(
                output,
                " * @property {{{}}} {}",
                js_type,
                to_camel_case(&field.name)
            )
            .unwrap();
        }
        writeln!(output, " */\n").unwrap();

        // Skip factory for non-parameterizable patterns (inlined at tree level)
        if !metadata.is_parameterizable(&pattern.name) {
            continue;
        }

        writeln!(output, "/**").unwrap();
        writeln!(output, " * Create a {} pattern node", pattern.name).unwrap();
        if pattern.is_generic {
            writeln!(output, " * @template T").unwrap();
        }
        writeln!(output, " * @param {{BrkClient}} client").unwrap();
        writeln!(output, " * @param {{string}} acc - Accumulated series name").unwrap();
        if pattern.is_templated() {
            writeln!(output, " * @param {{string}} disc - Discriminator suffix").unwrap();
        }
        let return_type = if pattern.is_generic {
            format!("{}<T>", pattern.name)
        } else {
            pattern.name.clone()
        };
        writeln!(output, " * @returns {{{}}}", return_type).unwrap();
        writeln!(output, " */").unwrap();

        if pattern.is_templated() {
            writeln!(
                output,
                "function create{}(client, acc, disc) {{",
                pattern.name
            )
            .unwrap();
        } else {
            writeln!(output, "function create{}(client, acc) {{", pattern.name).unwrap();
        }
        writeln!(output, "  return {{").unwrap();

        let syntax = JavaScriptSyntax;
        for field in &pattern.fields {
            generate_parameterized_field(output, &syntax, field, pattern, metadata, "    ");
        }

        writeln!(output, "  }};").unwrap();
        writeln!(output, "}}\n").unwrap();
    }
}
