/**
 * @typedef {Object} BitviewClientOptions
 * @property {string} baseUrl
 * @property {number} [timeout] - Request timeout in milliseconds, the download included. Default: 30000
 * @property {string|boolean} [browserCache] - Enable browser Cache API with default name (true), custom name (string), or disable (false). No effect in Node.js. Default: true
 * @property {number|boolean} [memCache] - LRU capacity of the in-memory parsed-response cache, which lets 304 responses skip the JSON parse: true/undefined → 1000, false/0 → disabled
 */

const _isBrowser = typeof window !== 'undefined' && 'caches' in window;
const _runIdle = (/** @type {VoidFunction} */ fn) => (globalThis.requestIdleCallback ?? setTimeout)(fn);
const _defaultBrowserCacheName = '__BRK_CLIENT__';
const _DEFAULT_MEM_CACHE_SIZE = 1000;

/** @template T @typedef {{ etag: string | null, value: T }} _MemEntry */
/** @type {Map<string, string>} */
const _camelNames = new Map();
/** `tx_count` → `txCount`; `avgFee_0` and `24h` stay as they are. @param {string} key */
const _camelName = (key) => {
  let name = _camelNames.get(key);
  if (name === undefined) {
    name = key.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());
    _camelNames.set(key, name);
  }
  return name;
};

/**
 * A response with camelCase field names, renamed once after parsing. `keep` says where the schema
 * has maps, whose keys are data and stay as sent with everything under them: `true` for the whole
 * value, or the fields (by wire name) that hold one; omitted, every key is a field name.
 * @param {*} value
 * @param {*} [keep]
 */
const _camelKeys = (value, keep) => {
  if (keep === true || value === null || typeof value !== 'object') return value;
  if (Array.isArray(value)) {
    for (let i = 0; i < value.length; i++) value[i] = _camelKeys(value[i], keep);
    return value;
  }
  let renamed = false;
  for (const key in value) {
    if (key.includes('_')) { renamed = true; break; }
  }
  if (!renamed) {
    for (const key in value) value[key] = _camelKeys(value[key], keep?.[key]);
    return value;
  }
  /** @type {Record<string, *>} */
  const out = {};
  for (const key in value) out[_camelName(key)] = _camelKeys(value[key], keep?.[key]);
  return out;
};

/**
 * @param {string|boolean|undefined} option
 * @returns {Promise<Cache | null>}
 */
const _openBrowserCache = (option) => {
  if (!_isBrowser || option === false) return Promise.resolve(null);
  const name = typeof option === 'string' ? option : _defaultBrowserCacheName;
  return caches.open(name).catch(() => null);
};

/**
 * @param {string} url
 * @returns {URL}
 */
const _parseBaseUrl = (url) => new URL(url, typeof location === 'undefined' ? undefined : location.href);

/**
 * A failed request: the HTTP status and the server's error code and message, when it sent them.
 */
class BitviewError extends Error {
  /**
   * @param {string} message
   * @param {number} [status]
   * @param {ErrorCode} [code]
   */
  constructor(message, status, code) {
    super(message);
    this.name = 'BitviewError';
    this.status = status;
    this.code = code;
  }
}

/**
 * The server's error body as a `BitviewError`; a body this client can't read keeps its text.
 * @param {Response} res
 * @returns {Promise<BitviewError>}
 */
const _responseError = async (res) => {
  const text = await res.text().catch(() => '');
  /** @type {Partial<ErrorDetail> | undefined} */
  let detail;
  try {
    detail = JSON.parse(text)?.error;
  } catch {}
  const message = typeof detail?.message === 'string' ? detail.message : text || `HTTP ${res.status}`;
  return new BitviewError(message, res.status, detail?.code);
};

// Date conversion, mirroring the server's indexes (`_CALENDARS`): UTC, from 2009-01-01.
const _EPOCH_MS = 1230768000000;
const _UNIT_MS = { second: 1000, day: 86400000 };

/** @param {string} index */
const _calendar = (index) => {
  const calendar = _CALENDARS.get(index);
  if (!calendar) throw new Error(`${index} is not a date-based index`);
  return calendar;
};

/**
 * The UTC start of bucket `i`; throws for an index without a date calendar.
 * @param {Index} index
 * @param {number} i
 * @returns {globalThis.Date}
 */
function indexToDate(index, i) {
  const [unit, size, phase] = _calendar(index);
  if (unit === 'month') return new Date(Date.UTC(2009, i === 0 ? 0 : i * size - phase, 1));
  return new Date(_EPOCH_MS + (i * size - phase) * _UNIT_MS[unit]);
}

/**
 * The floor index (latest index whose date is <= the given date).
 * Throws for dates before the index's first date (2009-01-01; day3's is 2008-12-31, week1's 2008-12-29).
 * @param {Index} index
 * @param {globalThis.Date} d
 * @returns {number}
 */
function dateToIndex(index, d) {
  const ms = d.getTime(), first = indexToDate(index, 0);
  if (!(ms >= first.getTime())) throw new RangeError(`Date is before the first ${index} date (${first.toISOString().slice(0, 10)})`);
  const [unit, size, phase] = _calendar(index);
  if (unit === 'month') return Math.floor(((d.getUTCFullYear() - 2009) * 12 + d.getUTCMonth() + phase) / size);
  return Math.floor((ms - _EPOCH_MS + phase * _UNIT_MS[unit]) / (size * _UNIT_MS[unit]));
}

/**
 * @template T
 * @param {SeriesData<T>} raw
 * @returns {DateSeriesData<T>}
 */
function _wrapSeriesData(raw) {
  const { index, start, end, data } = raw;
  const _dateBased = _CALENDARS.has(index);
  return /** @type {DateSeriesData<T>} */ ({
    ...raw,
    isDateBased: _dateBased,
    indexes() {
      /** @type {number[]} */
      const result = new Array(end - start);
      for (let i = 0; i < result.length; i++) result[i] = start + i;
      return result;
    },
    keys() {
      return this.indexes();
    },
    entries() {
      /** @type {Array<[number, T]>} */
      const result = new Array(data.length);
      for (let i = 0; i < data.length; i++) result[i] = [start + i, data[i]];
      return result;
    },
    toMap() {
      /** @type {Map<number, T>} */
      const map = new Map();
      for (let i = 0; i < data.length; i++) map.set(start + i, data[i]);
      return map;
    },
    *[Symbol.iterator]() {
      for (let i = 0; i < data.length; i++) yield /** @type {[number, T]} */ ([start + i, data[i]]);
    },
    // DateSeriesData methods (only meaningful for date-based indexes)
    dates() {
      /** @type {globalThis.Date[]} */
      const result = [];
      for (let i = start; i < end; i++) result.push(indexToDate(index, i));
      return result;
    },
    dateEntries() {
      /** @type {Array<[globalThis.Date, T]>} */
      const result = [];
      for (let i = 0; i < data.length; i++) result.push([indexToDate(index, start + i), data[i]]);
      return result;
    },
    toDateMap() {
      /** @type {Map<globalThis.Date, T>} */
      const map = new Map();
      for (let i = 0; i < data.length; i++) map.set(indexToDate(index, start + i), data[i]);
      return map;
    },
  });
}

/**
 * @template T
 * @typedef {Object} SeriesDataBase
 * @property {number} version
 * @property {Index} index
 * @property {string} type - Value type (e.g. "Ratio", "Sats", "Cents")
 * @property {number} start - Start index (inclusive)
 * @property {number} end - End index (exclusive)
 * @property {T[]} data - The series data (`null` where a value is missing or undefined)
 * @property {boolean} isDateBased
 * @property {() => number[]} indexes
 * @property {() => number[]} keys - Alias for indexes
 * @property {() => Array<[number, T]>} entries - Get [index, value] pairs
 * @property {() => Map<number, T>} toMap - Convert to Map<index, value>
 */

/** @template T @typedef {SeriesDataBase<T> & Iterable<[number, T]>} SeriesData */

/**
 * @template T
 * @typedef {Object} DateSeriesDataExtras
 * @property {() => globalThis.Date[]} dates - Get dates for each data point
 * @property {() => Array<[globalThis.Date, T]>} dateEntries
 * @property {() => Map<globalThis.Date, T>} toDateMap
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
 * @property {boolean} [memCache] - Use the parsed in-memory response cache. Set false for large one-shot reads.
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

/**
 * @template T
 * @param {BitviewClient} client
 * @param {string} name
 * @param {Index} index
 * @returns {DateSeriesEndpoint<T>}
 */
function _endpoint(client, name, index) {
  const path = `/api/series/${encodeURIComponent(name)}/${index}`;

  /**
   * Counted selections send `limit`: the server ends them after resolving a negative start.
   * @param {{ start?: number, end?: number, limit?: number }} range
   * @param {string} [format]
   * @returns {string}
   */
  const buildPath = ({ start, end, limit }, format) => {
    const params = new URLSearchParams();
    if (start !== undefined) params.set('start', String(start));
    if (end !== undefined) params.set('end', String(end));
    if (limit !== undefined) params.set('limit', String(limit));
    if (format) params.set('format', format);
    const query = params.toString();
    return query ? `${path}?${query}` : path;
  };

  /**
   * @param {{ start?: number, end?: number, limit?: number }} range
   * @returns {DateRangeBuilder<T>}
   */
  const rangeBuilder = (range) => ({
    fetch(arg, options) { return client._fetchSeriesData(buildPath(range), arg, options); },
    fetchCsv(options) { return client.getText(buildPath(range, 'csv'), options); },
    then(resolve, reject) { return this.fetch().then(resolve, reject); },
  });

  /**
   * @param {number} start
   * @returns {DateSkippedBuilder<T>}
   */
  const skippedBuilder = (start) => ({
    take(n) { return rangeBuilder({ start, limit: n }); },
    fetch(arg, options) { return client._fetchSeriesData(buildPath({ start }), arg, options); },
    fetchCsv(options) { return client.getText(buildPath({ start }, 'csv'), options); },
    then(resolve, reject) { return this.fetch().then(resolve, reject); },
  });

  /** @type {DateSeriesEndpoint<T>} */
  const endpoint = {
    get(idx) {
      if (idx instanceof Date) idx = dateToIndex(index, idx);
      return rangeBuilder({ start: idx, limit: 1 });
    },
    slice(start, end) {
      if (start instanceof Date) start = dateToIndex(index, start);
      if (end instanceof Date) end = dateToIndex(index, end);
      return rangeBuilder({ start, end });
    },
    first(n) { return rangeBuilder({ end: n }); },
    last(n) { return rangeBuilder(n === 0 ? { end: 0 } : { start: -n }); },
    skip(n) { return skippedBuilder(n); },
    fetch(arg, options) { return client._fetchSeriesData(buildPath({}), arg, options); },
    fetchCsv(options) { return client.getText(buildPath({}, 'csv'), options); },
    len() { return client.getSeriesLen(name, index); },
    version() { return client.getSeriesVersion(name, index); },
    then(resolve, reject) { return this.fetch().then(resolve, reject); },
    get path() { return path; },
  };

  return endpoint;
}

class BitviewClientBase {
  /**
   * @param {BitviewClientOptions|string} options
   */
  constructor(options) {
    const isString = typeof options === 'string';
    const rawUrl = isString ? options : options.baseUrl;
    this.baseUrl = rawUrl.endsWith('/') ? rawUrl.slice(0, -1) : rawUrl;
    const url = _parseBaseUrl(this.baseUrl);
    this.url = url.href.endsWith('/') ? url.href.slice(0, -1) : url.href;
    this.domain = url.hostname;
    this.timeout = isString ? 30_000 : (options.timeout ?? 30_000);
    /** @type {Promise<Cache | null>} */
    this._browserCachePromise = _openBrowserCache(isString ? undefined : options.browserCache);
    /** @type {Cache | null} */
    this._browserCache = null;
    this._browserCachePromise.then(c => this._browserCache = c);
    const memOpt = isString ? undefined : options.memCache;
    this._memCacheMax = memOpt === false || memOpt === 0
      ? 0
      : (typeof memOpt === 'number' ? memOpt : _DEFAULT_MEM_CACHE_SIZE);
    /** @type {Map<string, _MemEntry<unknown>>} */
    this._memCache = new Map();
  }

  /**
   * @template T
   * @param {string} key
   * @returns {_MemEntry<T> | undefined}
   */
  _memGet(key) {
    if (!this._memCacheMax) return undefined;
    const hit = this._memCache.get(key);
    if (!hit) return undefined;
    this._memCache.delete(key);
    this._memCache.set(key, hit);
    return /** @type {_MemEntry<T>} */ (hit);
  }

  /**
   * @param {string} key
   * @param {string | null} etag
   * @param {unknown} value
   */
  _memSet(key, etag, value) {
    if (!this._memCacheMax) return;
    if (this._memCache.has(key)) this._memCache.delete(key);
    else if (this._memCache.size >= this._memCacheMax) {
      const oldest = this._memCache.keys().next().value;
      if (oldest !== undefined) this._memCache.delete(oldest);
    }
    this._memCache.set(key, { etag, value });
  }

  /**
   * The request's timeout, joined with the caller's signal when one is given.
   * @param {AbortSignal} [signal]
   * @returns {AbortSignal}
   */
  _signal(signal) {
    const timeout = AbortSignal.timeout(this.timeout);
    return signal ? AbortSignal.any([timeout, signal]) : timeout;
  }

  /**
   * @param {string} path
   * @param {{ signal?: AbortSignal, cache?: boolean, etag?: string | null }} [options]
   * @returns {Promise<Response>}
   */
  async get(path, { signal, cache = true, etag } = {}) {
    const url = `${this.baseUrl}${path}`;
    /** @type {RequestInit} */
    const init = { signal: this._signal(signal) };
    // Let browsers manage HTTP revalidation; explicit validators bypass their HTTP cache.
    const revalidate = cache && etag && typeof location === 'undefined';
    if (revalidate) init.headers = { 'If-None-Match': etag };
    if (!cache) init.cache = 'no-store';
    const res = await fetch(url, init);
    if (!res.ok && !(revalidate && res.status === 304)) throw await _responseError(res);
    return res;
  }

  /**
   * Make a GET request with layered caching.
   *
   * Contract:
   * - The returned Promise resolves with the **freshest** value (post-revalidation) and rejects
   *   when that fails, even if a stale value was already delivered.
   * - `onValue` fires once with the freshest value, or twice if a stale snapshot
   *   could be shown first (stale-while-revalidate). On a 304 there is no second fire.
   * - Responses marked `Cache-Control: no-store` are never cached.
   *
   * Layers:
   * - L1 (memCache): in-memory parsed values keyed by body kind + URL, with their ETag. Lets 304s
   *   skip the parse entirely.
   * - L2 (browserCache): Cache API, survives reload and feeds onValue fast on cold start.
   *
   * @template T
   * @param {string} path
   * @param {'json' | 'text' | 'bytes'} kind - The body reader, part of the memCache key
   * @param {(res: Response) => Promise<T>} parse - Response body reader
   * @param {ClientFetchOptions<T>} [options]
   * @returns {Promise<T>}
   */
  async _getCached(path, kind, parse, { onValue, signal, cache = true, memCache = true } = {}) {
    if (!cache) {
      const res = await this.get(path, { signal, cache });
      const value = await parse(res);
      if (onValue) onValue(value);
      return value;
    }

    const url = `${this.baseUrl}${path}`;
    const key = `${kind} ${url}`;
    const useMemCache = memCache !== false;
    /** @type {_MemEntry<T> | undefined} */
    const memHit = useMemCache ? this._memGet(key) : undefined;
    const browserCache = this._browserCache;
    /** @param {Response} res */
    const storable = (res) => !/no-store/i.test(res.headers.get('Cache-Control') ?? '');
    /** Parse a fresh response, cache it unless `no-store`, and deliver it. @param {Response} res */
    const fresh = async (res) => {
      const keep = storable(res);
      const cloned = keep && browserCache ? res.clone() : null;
      const value = await parse(res);
      if (keep && useMemCache) this._memSet(key, res.headers.get('ETag'), value);
      if (cloned && browserCache) {
        const cacheStore = browserCache;
        _runIdle(() => cacheStore.put(url, cloned));
      }
      if (onValue) onValue(value);
      return value;
    };

    // L1 fast path: deliver from memCache, revalidate via network.
    // ETag match → zero parse, zero clone, zero cache write, no second onValue fire.
    if (memHit) {
      if (onValue) onValue(memHit.value);
      const res = await this.get(path, { signal, etag: memHit.etag });
      const netEtag = res.headers.get('ETag');
      if (res.status === 304 || (netEtag && netEtag === memHit.etag)) {
        await res.body?.cancel();
        return memHit.value;
      }
      return fresh(res);
    }

    // L1 miss: race browserCache (stale snapshot) vs network (fresh).
    let networkSettled = false;
    // What this request's stale snapshot delivered, if anything.
    let shown = /** @type {{ value: T } | null} */ (null);
    const stale = onValue && browserCache
      ? browserCache.match(url).then(async (res) => {
          if (!res || networkSettled || signal?.aborted) return;
          // Older clients stored `no-store` responses: drop them.
          if (!storable(res)) return void browserCache.delete(url);
          const value = await parse(res);
          if (networkSettled || signal?.aborted) return;
          if (useMemCache) this._memSet(key, res.headers.get('ETag'), value);
          shown = { value };
          onValue(value);
        }).catch(() => {})
      : null;

    let res;
    try {
      res = await this.get(path, { signal });
    } catch (e) {
      // The stale snapshot, if any, arrives before the rejection, never after.
      await stale;
      throw e;
    }
    networkSettled = true;
    const netEtag = res.headers.get('ETag');
    // memCache already holds this ETag (our stale snapshot or a concurrent request) → reuse, skip
    // the parse; deliver it unless this request already showed that very value.
    const populated = useMemCache ? /** @type {_MemEntry<T> | undefined} */ (this._memGet(key)) : undefined;
    if (populated && netEtag && netEtag === populated.etag) {
      await res.body?.cancel();
      if (onValue && shown?.value !== populated.value) onValue(populated.value);
      return populated.value;
    }
    return fresh(res);
  }

  /**
   * Cached and supports `onValue`. Field names come camelCased, map keys as sent (`keep`).
   * @template T
   * @param {string} path
   * @param {ClientFetchOptions<T>} [options]
   * @param {*} [keep] Where the response has maps, as `_camelKeys` takes it
   * @returns {Promise<T>}
   */
  getJson(path, options, keep) {
    return this._getCached(path, 'json', async (res) => _camelKeys(await res.json(), keep), options);
  }

  /**
   * Cached and supports `onValue`, same as `getJson`.
   * @param {string} path
   * @param {ClientFetchOptions<string>} [options]
   * @returns {Promise<string>}
   */
  getText(path, options) {
    return this._getCached(path, 'text', (res) => res.text(), options);
  }

  /**
   * Cached and supports `onValue`, same as `getJson`.
   * @param {string} path
   * @param {ClientFetchOptions<Uint8Array>} [options]
   * @returns {Promise<Uint8Array>}
   */
  getBytes(path, options) {
    return this._getCached(path, 'bytes', async (res) => new Uint8Array(await res.arrayBuffer()), options);
  }

  /**
   * POST responses are uncached and never invoke `onValue` — every call hits
   * the network with the same body and returns the upstream response.
   *
   * @param {string} path
   * @param {string} body
   * @param {{ signal?: AbortSignal }} [options]
   * @returns {Promise<Response>}
   */
  async post(path, body, { signal } = {}) {
    const url = `${this.baseUrl}${path}`;
    const res = await fetch(url, { method: 'POST', body, signal: this._signal(signal) });
    if (!res.ok) throw await _responseError(res);
    return res;
  }

  /**
   * @template T
   * @param {string} path
   * @param {string} body
   * @param {{ signal?: AbortSignal }} [options]
   * @param {*} [keep] Where the response has maps, as `_camelKeys` takes it
   * @returns {Promise<T>}
   */
  async postJson(path, body, options, keep) {
    const res = await this.post(path, body, options);
    return _camelKeys(await res.json(), keep);
  }

  /**
   * @param {string} path
   * @param {string} body
   * @param {{ signal?: AbortSignal }} [options]
   * @returns {Promise<string>}
   */
  async postText(path, body, options) {
    const res = await this.post(path, body, options);
    return res.text();
  }

  /**
   * @param {string} path
   * @param {string} body
   * @param {{ signal?: AbortSignal }} [options]
   * @returns {Promise<Uint8Array>}
   */
  async postBytes(path, body, options) {
    const res = await this.post(path, body, options);
    return new Uint8Array(await res.arrayBuffer());
  }

  /**
   * @template T
   * @param {string} path
   * @param {DateSeriesFetchArg<T>} [arg]
   * @param {ClientFetchOptions<DateSeriesData<T>>} [options]
   * @returns {Promise<DateSeriesData<T>>}
   */
  async _fetchSeriesData(path, arg, options) {
    const requestOptions = typeof arg === 'function'
      ? { ...(options ?? {}), onValue: arg }
      : { ...(arg ?? {}), ...(options ?? {}) };
    const onValue = requestOptions.onValue;
    const wrappedOnValue = onValue ? (/** @type {SeriesData<T>} */ raw) => onValue(_wrapSeriesData(raw)) : undefined;
    const raw = await this.getJson(path, { ...requestOptions, onValue: wrappedOnValue });
    return _wrapSeriesData(raw);
  }
}

/**
 * Materialize and replace a lazy object property.
 * @template T
 * @param {object} owner
 * @param {string} name
 * @param {() => T} init
 * @returns {T}
 */
function _lazy(owner, name, init) {
  const value = init();
  Object.defineProperty(owner, name, { value, writable: true, enumerable: true, configurable: true });
  return value;
}

const _MASK_64 = 0xffffffffffffffffn;
// RapidHash v3's secrets 1 and 2: the only ones payloads of at most 65 bytes reach.
const _RAPIDHASH_SECRET_1 = 0x8bb84b93962eacc9n;
const _RAPIDHASH_SECRET_2 = 0x4b33a62ed433d4a3n;
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
  return _u64(seed ^ _rapidMix(seed ^ _RAPIDHASH_SECRET_2, _RAPIDHASH_SECRET_1));
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
    } else {
      a ^= (BigInt(bytes[0]) << 45n) | BigInt(bytes[length - 1]);
      b ^= BigInt(bytes[length >> 1]);
    }
    remainder = BigInt(length);
  } else {
    seed = _rapidMix(_readU64(bytes, 0) ^ _RAPIDHASH_SECRET_2, _readU64(bytes, 8) ^ seed);
    if (length > 32) {
      seed = _rapidMix(_readU64(bytes, 16) ^ _RAPIDHASH_SECRET_2, _readU64(bytes, 24) ^ seed);
      if (length > 48) {
        seed = _rapidMix(_readU64(bytes, 32) ^ _RAPIDHASH_SECRET_1, _readU64(bytes, 40) ^ seed);
        if (length > 64) {
          seed = _rapidMix(_readU64(bytes, 48) ^ _RAPIDHASH_SECRET_1, _readU64(bytes, 56) ^ seed);
        }
      }
    }
    remainder = BigInt(length);
    a ^= _readU64(bytes, length - 16) ^ remainder;
    b ^= _readU64(bytes, length - 8);
  }

  a ^= _RAPIDHASH_SECRET_1;
  b ^= seed;
  [a, b] = _rapidMum(a, b);
  return _rapidMix(a ^ 0xaaaaaaaaaaaaaaaan, b ^ _RAPIDHASH_SECRET_1 ^ remainder);
}

/** @param {number} nibbles */
function _validateHashPrefixNibbles(nibbles) {
  if (!Number.isInteger(nibbles) || nibbles < 1 || nibbles > 16) {
    throw new Error("Expected hash-prefix length from 1 to 16 hex nibbles");
  }
}

/**
 * @param {AddressOutputType} addressType
 * @param {Uint8Array | ArrayBuffer | ArrayBufferView | number[]} payload
 */
function _validateAddressPayloadForType(addressType, payload) {
  const length = _asUint8Array(payload).length;
  const expected = _ADDRESS_PAYLOAD_LENGTHS.get(addressType);
  if (expected === undefined) {
    throw new Error(`Unsupported address type for address payload hash-prefix: ${addressType}`);
  }
  if (length !== expected) {
    throw new Error(`Expected ${addressType} address payload length ${expected} bytes`);
  }
}

/**
 * Compute the RapidHash v3 hash-prefix used by `/api/address/hash-prefix/{address_type}/{prefix}`.
 * @param {Uint8Array | ArrayBuffer | ArrayBufferView | number[]} payload
 * @param {number} nibbles - Prefix length from 1 to 16 hex nibbles
 * @returns {string}
 */
function addressPayloadHashPrefix(payload, nibbles) {
  _validateHashPrefixNibbles(nibbles);
  return _rapidHashV3(payload).toString(16).padStart(16, "0").slice(0, nibbles);
}

