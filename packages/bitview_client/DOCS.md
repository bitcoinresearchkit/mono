# Table of Contents

* [bitview\_client](#bitview_client)
  * [BitviewError](#bitview_client.BitviewError)
  * [SeriesData](#bitview_client.SeriesData)
  * [DateSeriesData](#bitview_client.DateSeriesData)
  * [SeriesEndpoint](#bitview_client.SeriesEndpoint)
  * [DateSeriesEndpoint](#bitview_client.DateSeriesEndpoint)
  * [BitviewClient](#bitview_client.BitviewClient)
    * [VERSION](#bitview_client.BitviewClient.VERSION)
    * [INDEXES](#bitview_client.BitviewClient.INDEXES)
    * [POOL\_SLUG\_TO\_NAME](#bitview_client.BitviewClient.POOL_SLUG_TO_NAME)
    * [TERM\_NAMES](#bitview_client.BitviewClient.TERM_NAMES)
    * [HALVING\_EPOCH\_NAMES](#bitview_client.BitviewClient.HALVING_EPOCH_NAMES)
    * [VINTAGE\_NAMES](#bitview_client.BitviewClient.VINTAGE_NAMES)
    * [ENTRY\_NAMES](#bitview_client.BitviewClient.ENTRY_NAMES)
    * [SPENDABLE\_TYPE\_NAMES](#bitview_client.BitviewClient.SPENDABLE_TYPE_NAMES)
    * [AGE\_RANGE\_NAMES](#bitview_client.BitviewClient.AGE_RANGE_NAMES)
    * [AMOUNT\_RANGE\_NAMES](#bitview_client.BitviewClient.AMOUNT_RANGE_NAMES)
    * [PROFITABILITY\_RANGE\_NAMES](#bitview_client.BitviewClient.PROFITABILITY_RANGE_NAMES)
    * [OP\_RETURN\_KIND\_NAMES](#bitview_client.BitviewClient.OP_RETURN_KIND_NAMES)
    * [\_\_init\_\_](#bitview_client.BitviewClient.__init__)
    * [series](#bitview_client.BitviewClient.series)
    * [series\_endpoint](#bitview_client.BitviewClient.series_endpoint)
    * [index\_to\_date](#bitview_client.BitviewClient.index_to_date)
    * [date\_to\_index](#bitview_client.BitviewClient.date_to_index)
    * [address\_payload\_hash\_prefix](#bitview_client.BitviewClient.address_payload_hash_prefix)
    * [get\_address\_payload\_hash\_prefix\_matches](#bitview_client.BitviewClient.get_address_payload_hash_prefix_matches)
    * [get\_health](#bitview_client.BitviewClient.get_health)
    * [get\_version](#bitview_client.BitviewClient.get_version)
    * [get\_sync\_status](#bitview_client.BitviewClient.get_sync_status)
    * [get\_disk\_usage](#bitview_client.BitviewClient.get_disk_usage)
    * [get\_series\_tree](#bitview_client.BitviewClient.get_series_tree)
    * [get\_series\_count](#bitview_client.BitviewClient.get_series_count)
    * [list\_indexes](#bitview_client.BitviewClient.list_indexes)
    * [list\_series](#bitview_client.BitviewClient.list_series)
    * [search\_series](#bitview_client.BitviewClient.search_series)
    * [get\_series\_info](#bitview_client.BitviewClient.get_series_info)
    * [get\_series](#bitview_client.BitviewClient.get_series)
    * [get\_series\_data](#bitview_client.BitviewClient.get_series_data)
    * [get\_series\_latest](#bitview_client.BitviewClient.get_series_latest)
    * [get\_series\_len](#bitview_client.BitviewClient.get_series_len)
    * [get\_series\_version](#bitview_client.BitviewClient.get_series_version)
    * [get\_series\_bulk](#bitview_client.BitviewClient.get_series_bulk)
    * [list\_urpd\_cohorts](#bitview_client.BitviewClient.list_urpd_cohorts)
    * [get\_urpd\_latest](#bitview_client.BitviewClient.get_urpd_latest)
    * [get\_urpd](#bitview_client.BitviewClient.get_urpd)
    * [get\_difficulty\_adjustment](#bitview_client.BitviewClient.get_difficulty_adjustment)
    * [get\_prices](#bitview_client.BitviewClient.get_prices)
    * [get\_historical\_price](#bitview_client.BitviewClient.get_historical_price)
    * [get\_address\_hash\_prefix\_matches](#bitview_client.BitviewClient.get_address_hash_prefix_matches)
    * [get\_address](#bitview_client.BitviewClient.get_address)
    * [get\_address\_txs](#bitview_client.BitviewClient.get_address_txs)
    * [get\_address\_confirmed\_txs](#bitview_client.BitviewClient.get_address_confirmed_txs)
    * [get\_address\_confirmed\_txs\_after](#bitview_client.BitviewClient.get_address_confirmed_txs_after)
    * [get\_address\_mempool\_txs](#bitview_client.BitviewClient.get_address_mempool_txs)
    * [get\_address\_utxos](#bitview_client.BitviewClient.get_address_utxos)
    * [validate\_address](#bitview_client.BitviewClient.validate_address)
    * [get\_block](#bitview_client.BitviewClient.get_block)
    * [get\_block\_v1](#bitview_client.BitviewClient.get_block_v1)
    * [get\_block\_header](#bitview_client.BitviewClient.get_block_header)
    * [get\_block\_hash\_by\_height](#bitview_client.BitviewClient.get_block_hash_by_height)
    * [get\_block\_by\_timestamp](#bitview_client.BitviewClient.get_block_by_timestamp)
    * [get\_block\_raw](#bitview_client.BitviewClient.get_block_raw)
    * [get\_block\_status](#bitview_client.BitviewClient.get_block_status)
    * [get\_block\_tip\_height](#bitview_client.BitviewClient.get_block_tip_height)
    * [get\_block\_tip\_hash](#bitview_client.BitviewClient.get_block_tip_hash)
    * [get\_block\_txid](#bitview_client.BitviewClient.get_block_txid)
    * [get\_block\_txids](#bitview_client.BitviewClient.get_block_txids)
    * [get\_block\_txs](#bitview_client.BitviewClient.get_block_txs)
    * [get\_block\_txs\_from\_index](#bitview_client.BitviewClient.get_block_txs_from_index)
    * [get\_blocks](#bitview_client.BitviewClient.get_blocks)
    * [get\_blocks\_from\_height](#bitview_client.BitviewClient.get_blocks_from_height)
    * [get\_blocks\_v1](#bitview_client.BitviewClient.get_blocks_v1)
    * [get\_blocks\_v1\_from\_height](#bitview_client.BitviewClient.get_blocks_v1_from_height)
    * [get\_pools](#bitview_client.BitviewClient.get_pools)
    * [get\_pool\_ranking](#bitview_client.BitviewClient.get_pool_ranking)
    * [get\_pool](#bitview_client.BitviewClient.get_pool)
    * [get\_pools\_hashrate](#bitview_client.BitviewClient.get_pools_hashrate)
    * [get\_pools\_hashrate\_by\_period](#bitview_client.BitviewClient.get_pools_hashrate_by_period)
    * [get\_pool\_hashrate](#bitview_client.BitviewClient.get_pool_hashrate)
    * [get\_pool\_blocks](#bitview_client.BitviewClient.get_pool_blocks)
    * [get\_pool\_blocks\_from\_height](#bitview_client.BitviewClient.get_pool_blocks_from_height)
    * [get\_hashrate](#bitview_client.BitviewClient.get_hashrate)
    * [get\_hashrate\_by\_period](#bitview_client.BitviewClient.get_hashrate_by_period)
    * [get\_difficulty\_adjustments](#bitview_client.BitviewClient.get_difficulty_adjustments)
    * [get\_difficulty\_adjustments\_by\_period](#bitview_client.BitviewClient.get_difficulty_adjustments_by_period)
    * [get\_reward\_stats](#bitview_client.BitviewClient.get_reward_stats)
    * [get\_block\_fees](#bitview_client.BitviewClient.get_block_fees)
    * [get\_block\_rewards](#bitview_client.BitviewClient.get_block_rewards)
    * [get\_block\_fee\_rates](#bitview_client.BitviewClient.get_block_fee_rates)
    * [get\_block\_fee\_rates\_precise](#bitview_client.BitviewClient.get_block_fee_rates_precise)
    * [get\_block\_sizes\_weights](#bitview_client.BitviewClient.get_block_sizes_weights)
    * [get\_mempool\_blocks](#bitview_client.BitviewClient.get_mempool_blocks)
    * [get\_recommended\_fees](#bitview_client.BitviewClient.get_recommended_fees)
    * [get\_precise\_fees](#bitview_client.BitviewClient.get_precise_fees)
    * [get\_mempool](#bitview_client.BitviewClient.get_mempool)
    * [get\_mempool\_txids](#bitview_client.BitviewClient.get_mempool_txids)
    * [get\_mempool\_recent](#bitview_client.BitviewClient.get_mempool_recent)
    * [get\_replacements](#bitview_client.BitviewClient.get_replacements)
    * [get\_fullrbf\_replacements](#bitview_client.BitviewClient.get_fullrbf_replacements)
    * [get\_mempool\_block](#bitview_client.BitviewClient.get_mempool_block)
    * [get\_mempool\_block\_diff](#bitview_client.BitviewClient.get_mempool_block_diff)
    * [get\_txid\_by\_index](#bitview_client.BitviewClient.get_txid_by_index)
    * [get\_cpfp](#bitview_client.BitviewClient.get_cpfp)
    * [get\_tx\_rbf](#bitview_client.BitviewClient.get_tx_rbf)
    * [get\_tx](#bitview_client.BitviewClient.get_tx)
    * [get\_tx\_hex](#bitview_client.BitviewClient.get_tx_hex)
    * [get\_tx\_merkleblock\_proof](#bitview_client.BitviewClient.get_tx_merkleblock_proof)
    * [get\_tx\_merkle\_proof](#bitview_client.BitviewClient.get_tx_merkle_proof)
    * [get\_tx\_outspend](#bitview_client.BitviewClient.get_tx_outspend)
    * [get\_tx\_outspends](#bitview_client.BitviewClient.get_tx_outspends)
    * [get\_tx\_raw](#bitview_client.BitviewClient.get_tx_raw)
    * [get\_tx\_status](#bitview_client.BitviewClient.get_tx_status)
    * [get\_transaction\_times](#bitview_client.BitviewClient.get_transaction_times)
    * [post\_tx](#bitview_client.BitviewClient.post_tx)
    * [get\_utxo\_set\_latest](#bitview_client.BitviewClient.get_utxo_set_latest)
    * [get\_utxo\_set](#bitview_client.BitviewClient.get_utxo_set)
    * [get\_utxo\_set\_diff](#bitview_client.BitviewClient.get_utxo_set_diff)
    * [get\_live\_price](#bitview_client.BitviewClient.get_live_price)
    * [get\_oracle\_histogram\_payments\_live](#bitview_client.BitviewClient.get_oracle_histogram_payments_live)
    * [get\_oracle\_histogram\_payments](#bitview_client.BitviewClient.get_oracle_histogram_payments)
    * [get\_oracle\_histogram\_outputs\_live](#bitview_client.BitviewClient.get_oracle_histogram_outputs_live)
    * [get\_oracle\_histogram\_outputs](#bitview_client.BitviewClient.get_oracle_histogram_outputs)
    * [get\_openapi](#bitview_client.BitviewClient.get_openapi)
    * [get\_api](#bitview_client.BitviewClient.get_api)

<a id="bitview_client"></a>

# bitview\_client

<a id="bitview_client.BitviewError"></a>

## BitviewError Objects

```python
class BitviewError(Exception)
```

A failed request: the HTTP status and the server's error code and message, when it sent them.

<a id="bitview_client.SeriesData"></a>

## SeriesData Objects

```python
@dataclass
class SeriesData(Generic[T])
```

Always int-indexed; a value is None where it is missing or undefined.

<a id="bitview_client.DateSeriesData"></a>

## DateSeriesData Objects

```python
@dataclass
class DateSeriesData(SeriesData[T])
```

<a id="bitview_client.SeriesEndpoint"></a>

## SeriesEndpoint Objects

```python
class SeriesEndpoint(Generic[T])
```

Builder for series endpoint queries with int-based indexing.

Examples:
    data = endpoint.fetch()
    data = endpoint[5].fetch()
    data = endpoint[:10].fetch()
    data = endpoint.head(20).fetch()
    data = endpoint.skip(100).take(10).fetch()

<a id="bitview_client.DateSeriesEndpoint"></a>

## DateSeriesEndpoint Objects

```python
class DateSeriesEndpoint(SeriesEndpoint[T])
```

Builder for series endpoint queries with date-based indexing.

Accepts dates in __getitem__ and returns DateSeriesData from fetch().

Examples:
    data = endpoint[date(2020, 1, 1)].fetch()
    data = endpoint[date(2020, 1, 1):date(2023, 1, 1)].fetch()

<a id="bitview_client.BitviewClient"></a>

## BitviewClient Objects

```python
class BitviewClient(BitviewClientBase)
```

A method per API endpoint, and the typed series tree.

<a id="bitview_client.BitviewClient.VERSION"></a>

#### VERSION

<a id="bitview_client.BitviewClient.INDEXES"></a>

#### INDEXES

<a id="bitview_client.BitviewClient.POOL_SLUG_TO_NAME"></a>

#### POOL\_SLUG\_TO\_NAME

<a id="bitview_client.BitviewClient.TERM_NAMES"></a>

#### TERM\_NAMES

<a id="bitview_client.BitviewClient.HALVING_EPOCH_NAMES"></a>

#### HALVING\_EPOCH\_NAMES

<a id="bitview_client.BitviewClient.VINTAGE_NAMES"></a>

#### VINTAGE\_NAMES

<a id="bitview_client.BitviewClient.ENTRY_NAMES"></a>

#### ENTRY\_NAMES

<a id="bitview_client.BitviewClient.SPENDABLE_TYPE_NAMES"></a>

#### SPENDABLE\_TYPE\_NAMES

<a id="bitview_client.BitviewClient.AGE_RANGE_NAMES"></a>

#### AGE\_RANGE\_NAMES

<a id="bitview_client.BitviewClient.AMOUNT_RANGE_NAMES"></a>

#### AMOUNT\_RANGE\_NAMES

<a id="bitview_client.BitviewClient.PROFITABILITY_RANGE_NAMES"></a>

#### PROFITABILITY\_RANGE\_NAMES

<a id="bitview_client.BitviewClient.OP_RETURN_KIND_NAMES"></a>

#### OP\_RETURN\_KIND\_NAMES

<a id="bitview_client.BitviewClient.__init__"></a>

#### \_\_init\_\_

```python
def __init__(base_url: str = 'http://localhost:3110', timeout: float = 30.0)
```

<a id="bitview_client.BitviewClient.series"></a>

#### series

```python
@cached_property
def series() -> SeriesTree
```

<a id="bitview_client.BitviewClient.series_endpoint"></a>

#### series\_endpoint

```python
def series_endpoint(
        series: str,
        index: Index) -> Union[SeriesEndpoint[Any], DateSeriesEndpoint[Any]]
```

A series by a name known only at runtime; `series` has the typed paths.

<a id="bitview_client.BitviewClient.index_to_date"></a>

#### index\_to\_date

```python
def index_to_date(index: Index, i: int) -> Union[date, datetime]
```

The start of bucket `i`: a UTC datetime for sub-daily indexes, a date otherwise.

<a id="bitview_client.BitviewClient.date_to_index"></a>

#### date\_to\_index

```python
def date_to_index(index: Index, d: Union[date, datetime]) -> int
```

The floor index (latest index whose date is <= the given date).

<a id="bitview_client.BitviewClient.address_payload_hash_prefix"></a>

#### address\_payload\_hash\_prefix

```python
@staticmethod
def address_payload_hash_prefix(payload: Union[bytes, bytearray, memoryview],
                                nibbles: int) -> str
```

Compute the RapidHash v3 hash-prefix for raw address payload bytes.

<a id="bitview_client.BitviewClient.get_address_payload_hash_prefix_matches"></a>

#### get\_address\_payload\_hash\_prefix\_matches

```python
def get_address_payload_hash_prefix_matches(
        address_type: OutputType, payload: Union[bytes, bytearray, memoryview],
        nibbles: int) -> AddressHashPrefixMatches
```

Fetch address hash-prefix matches from raw payload bytes matching address_type length.

<a id="bitview_client.BitviewClient.get_health"></a>

#### get\_health

```python
def get_health() -> Health
```

Health check.

Local health and query-readiness check. Returns server identity, uptime, and a coherent local sync snapshot without a bitcoind round-trip. Reads the published prefix during processing; an empty index returns 503 `state_updating` at once. Responses are not cached. For chain-tip catch-up, request `GET /api/server/sync`.

Endpoint: `GET /health`

<a id="bitview_client.BitviewClient.get_version"></a>

#### get\_version

```python
def get_version() -> ServerVersion
```

API version

Endpoint: `GET /version`

<a id="bitview_client.BitviewClient.get_sync_status"></a>

#### get\_sync\_status

```python
def get_sync_status() -> SyncStatus
```

Sync status.

Returns a coherent local index snapshot and a separately observed Bitcoin Core tip height. The two heights can differ during indexing or a reorg. Conditional requests refresh these observations before validation.

Endpoint: `GET /api/server/sync`

<a id="bitview_client.BitviewClient.get_disk_usage"></a>

#### get\_disk\_usage

```python
def get_disk_usage() -> DiskUsage
```

Disk usage.

Returns allocated file bytes (file lengths on non-Unix platforms) for BRK and Bitcoin data. Each request scans both trees; these are independent observations, not an atomic filesystem snapshot. Conditional requests validate the newly observed totals. Directory-link cycles and excessive nesting fail without returning partial totals.

Endpoint: `GET /api/server/disk`

<a id="bitview_client.BitviewClient.get_series_tree"></a>

#### get\_series\_tree

```python
def get_series_tree() -> TreeNode
```

Series catalog.

The available series as a tree of categories and subcategories.

Endpoint: `GET /api/series`

<a id="bitview_client.BitviewClient.get_series_count"></a>

#### get\_series\_count

```python
def get_series_count() -> DetailedSeriesCount
```

Series count.

Returns the number of series available per index type.

Endpoint: `GET /api/series/count`

<a id="bitview_client.BitviewClient.list_indexes"></a>

#### list\_indexes

```python
def list_indexes() -> List[Index]
```

List available indexes.

Returns every index a series can be read at, by the name series routes take.

Endpoint: `GET /api/series/indexes`

<a id="bitview_client.BitviewClient.list_series"></a>

#### list\_series

```python
def list_series(page: Optional[int] = None,
                per_page: Optional[int] = None) -> PaginatedSeries
```

Series list.

Paginated flat list of all available series names.

Endpoint: `GET /api/series/list`

<a id="bitview_client.BitviewClient.search_series"></a>

#### search\_series

```python
def search_series(q: str, limit: Optional[Limit] = None) -> List[SeriesName]
```

Search series.

Search series by name or descriptive terms. Results prioritize whole query words in names, then descriptions, then fuzzy names, then fuzzy descriptions. Word order does not matter. Descriptions provide cohort terminology and formulas. The decoded q parameter is limited to 1024 UTF-8 bytes.

Endpoint: `GET /api/series/search`

<a id="bitview_client.BitviewClient.get_series_info"></a>

#### get\_series\_info

```python
def get_series_info(series: SeriesName) -> SeriesInfo
```

Get series info.

Returns the optional description, supported indexes, the indexes whose values can be null, the value type, and the optional unit (what the value type measures) for the specified series. The decoded series name is limited to 1024 UTF-8 bytes.

Endpoint: `GET /api/series/{series}`

<a id="bitview_client.BitviewClient.get_series"></a>

#### get\_series

```python
def get_series(series: SeriesName,
               index: Index,
               start: Optional[RangeIndex] = None,
               end: Optional[RangeIndex] = None,
               limit: Optional[Limit] = None,
               format: Optional[Format] = None) -> Union[AnySeriesData, str]
```

Get series data.

Rows selected by a range of positions, dates or timestamps, as JSON or CSV.

Endpoint: `GET /api/series/{series}/{index}`

<a id="bitview_client.BitviewClient.get_series_data"></a>

#### get\_series\_data

```python
def get_series_data(series: SeriesName,
                    index: Index,
                    start: Optional[RangeIndex] = None,
                    end: Optional[RangeIndex] = None,
                    limit: Optional[Limit] = None,
                    format: Optional[Format] = None) -> Union[List[Any], str]
```

Get raw series data.

The data array without the `SeriesData` wrapper. Supports the same range and format parameters as `GET /api/series/{series}/{index}`.

Endpoint: `GET /api/series/{series}/{index}/data`

<a id="bitview_client.BitviewClient.get_series_latest"></a>

#### get\_series\_latest

```python
def get_series_latest(series: SeriesName, index: Index) -> Any
```

Get latest series value.

Returns the single most recent value for a series, unwrapped (not inside a SeriesData object).

Endpoint: `GET /api/series/{series}/{index}/latest`

<a id="bitview_client.BitviewClient.get_series_len"></a>

#### get\_series\_len

```python
def get_series_len(series: SeriesName, index: Index) -> int
```

Get series data length

Endpoint: `GET /api/series/{series}/{index}/len`

<a id="bitview_client.BitviewClient.get_series_version"></a>

#### get\_series\_version

```python
def get_series_version(series: SeriesName, index: Index) -> Version
```

Get series version.

Returns the vector's schema/computation version, not its length or latest update. Appends and reorgs do not by themselves change this version.

Endpoint: `GET /api/series/{series}/{index}/version`

<a id="bitview_client.BitviewClient.get_series_bulk"></a>

#### get\_series\_bulk

```python
def get_series_bulk(
        series: SeriesList,
        index: Index,
        start: Optional[RangeIndex] = None,
        end: Optional[RangeIndex] = None,
        limit: Optional[Limit] = None,
        format: Optional[Format] = None) -> Union[List[AnySeriesData], str]
```

Bulk series data.

Fetch multiple series in a single request. Supports filtering by index and date range. Returns an array of SeriesData objects. For a single series, use `get_series` instead.

Endpoint: `GET /api/series/bulk`

<a id="bitview_client.BitviewClient.list_urpd_cohorts"></a>

#### list\_urpd\_cohorts

```python
def list_urpd_cohorts() -> List[UrpdCohort]
```

Available URPD cohorts.

Every URPD cohort once a UTXO set is published (none before): the age aggregates (`all`, `sth`, `lth`, `age_under_4m`, ...), then the age bands from the youngest (`age_under_1h`, `age_1h_to_1d`, ...).

Endpoint: `GET /api/urpd`

<a id="bitview_client.BitviewClient.get_urpd_latest"></a>

#### get\_urpd\_latest

```python
def get_urpd_latest(cohort: UrpdCohort,
                    agg: Optional[UrpdAggregation] = None,
                    weight: Optional[UrpdWeight] = None) -> Urpd
```

Latest URPD.

URPD for the latest published block; `date` echoes the date served. Fields, units, price-grid rounding and weighting as in `GET /api/urpd/{cohort}/{point}`.

Endpoint: `GET /api/urpd/{cohort}`

<a id="bitview_client.BitviewClient.get_urpd"></a>

#### get\_urpd

```python
def get_urpd(cohort: UrpdCohort,
             point: HeightOrDate,
             agg: Optional[UrpdAggregation] = None,
             weight: Optional[UrpdWeight] = None) -> Urpd
```

URPD at block height or date.

URPD for a cohort at a block height or the last block of a UTC day. Returns `{ height, hash, date, cohort, weight, aggregation, close, supply, buckets }` where `buckets` is columns, `{ price, supply, realized_cap, unrealized_pnl }`, one entry per bucket in ascending price: the same block header and columnar layout as `GET /api/utxo-set/{point}`. `close`, `price`, `realized_cap`, and `unrealized_pnl` are USD; `supply` is BTC. `unrealized_pnl` can be negative. Bucket `realized_cap` and `unrealized_pnl` value each coin at its creation price on the price grid (four significant digits of the price in cents, within 0.05% of the exact price), rounded to the cent per bucket, so the buckets need not sum to the realized cap series. With a `wakefulness` or `mobility` `weight`, `supply` is the weighted supply (whole satoshis) and the bucket values follow it.

Endpoint: `GET /api/urpd/{cohort}/{point}`

<a id="bitview_client.BitviewClient.get_difficulty_adjustment"></a>

#### get\_difficulty\_adjustment

```python
def get_difficulty_adjustment() -> DifficultyAdjustment
```

Difficulty adjustment.

Get current difficulty adjustment progress and estimates.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustment)*

Endpoint: `GET /api/v1/difficulty-adjustment`

<a id="bitview_client.BitviewClient.get_prices"></a>

#### get\_prices

```python
def get_prices() -> PricePoint
```

Current BTC price.

Returns bitcoin latest price (on-chain derived, USD only).

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-price)*

Endpoint: `GET /api/v1/prices`

<a id="bitview_client.BitviewClient.get_historical_price"></a>

#### get\_historical\_price

```python
def get_historical_price(
        timestamp: Optional[Timestamp] = None,
        currency: Optional[Currency] = None) -> HistoricalPrice
```

Historical price.

Completed four-hour BTC/USD closes, newest first as mempool.space lists them, labeled by interval end, from the first priced interval (2010-08-16). With a UNIX timestamp, returns the latest nonempty completed close at or before it; before the first close returns an empty list. The current partial interval is excluded. USD only (`currency` accepts `USD`); exchangeRates is empty.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-historical-price)*

Endpoint: `GET /api/v1/historical-price`

<a id="bitview_client.BitviewClient.get_address_hash_prefix_matches"></a>

#### get\_address\_hash\_prefix\_matches

```python
def get_address_hash_prefix_matches(address_type: OutputType,
                                    prefix: str) -> AddressHashPrefixMatches
```

Address hash-prefix matches.

Find addresses by address type and by the first 1-16 hex nibbles of RapidHash v3 over the raw address payload bytes. Intended for privacy-preserving client-side wallet discovery without sending raw addresses or xpubs. Fetch metadata with `GET /api/address/{address}`.

Endpoint: `GET /api/address/hash-prefix/{address_type}/{prefix}`

<a id="bitview_client.BitviewClient.get_address"></a>

#### get\_address

```python
def get_address(address: Address) -> AddressStats
```

Address information.

Balance and transaction counts. Supports P2PKH, P2SH, P2WPKH, P2WSH, P2TR and P2A addresses, and P2PK public keys in hex (33 or 65 bytes).

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address)*

Endpoint: `GET /api/address/{address}`

<a id="bitview_client.BitviewClient.get_address_txs"></a>

#### get\_address\_txs

```python
def get_address_txs(address: Address,
                    after_txid: Optional[Txid] = None,
                    max_txs: Optional[int] = None) -> List[Transaction]
```

Address transactions.

Get transaction history for an address, newest first: up to `max_txs` (50 by default and at most), mempool transactions first and confirmed ones filling the rest. To continue, pass the last returned txid as `after_txid`, or request `GET /api/address/{address}/txs/chain/{after_txid}` for confirmed history.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions)*

Endpoint: `GET /api/address/{address}/txs`

<a id="bitview_client.BitviewClient.get_address_confirmed_txs"></a>

#### get\_address\_confirmed\_txs

```python
def get_address_confirmed_txs(address: Address,
                              max_txs: Optional[int] = None
                              ) -> List[Transaction]
```

Address confirmed transactions.

Get the newest confirmed transactions for an address, up to `max_txs` (25 by default and at most). For pagination, request `GET /api/address/{address}/txs/chain/{after_txid}` with the last returned txid.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-chain)*

Endpoint: `GET /api/address/{address}/txs/chain`

<a id="bitview_client.BitviewClient.get_address_confirmed_txs_after"></a>

#### get\_address\_confirmed\_txs\_after

```python
def get_address_confirmed_txs_after(
        address: Address,
        after_txid: Txid,
        max_txs: Optional[int] = None) -> List[Transaction]
```

Address confirmed transactions (paginated).

Get the next confirmed transactions strictly older than `after_txid`, up to `max_txs` (25 by default and at most); a txid outside the address's history answers `[]` (Esplora-canonical pagination form, matches mempool.space).

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-chain)*

Endpoint: `GET /api/address/{address}/txs/chain/{after_txid}`

<a id="bitview_client.BitviewClient.get_address_mempool_txs"></a>

#### get\_address\_mempool\_txs

```python
def get_address_mempool_txs(address: Address,
                            max_txs: Optional[int] = None
                            ) -> List[Transaction]
```

Address mempool transactions.

Get unconfirmed transactions for an address from the mempool, newest first, up to `max_txs` (50 by default and at most).

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-mempool)*

Endpoint: `GET /api/address/{address}/txs/mempool`

<a id="bitview_client.BitviewClient.get_address_utxos"></a>

#### get\_address\_utxos

```python
def get_address_utxos(address: Address) -> List[Utxo]
```

Address UTXOs.

Get unspent transaction outputs (UTXOs) for an address, including unconfirmed ones; outputs that mempool transactions spend are left out.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-utxo)*

Endpoint: `GET /api/address/{address}/utxo`

<a id="bitview_client.BitviewClient.validate_address"></a>

#### validate\_address

```python
def validate_address(address: str) -> AddressValidation
```

Validate address.

Validate a Bitcoin address and get information about its type and scriptPubKey. Returns `isvalid: false` with an error message for invalid addresses; a string that isn't 2 to 130 letters and digits answers 400.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-validate)*

Endpoint: `GET /api/v1/validate-address/{address}`

<a id="bitview_client.BitviewClient.get_block"></a>

#### get\_block

```python
def get_block(hash: BlockHash) -> BlockInfo
```

Block information.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block)*

Endpoint: `GET /api/block/{hash}`

<a id="bitview_client.BitviewClient.get_block_v1"></a>

#### get\_block\_v1

```python
def get_block_v1(hash: BlockHash) -> BlockInfoV1
```

Block (v1).

Returns block details with extras by hash.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-v1)*

Endpoint: `GET /api/v1/block/{hash}`

<a id="bitview_client.BitviewClient.get_block_header"></a>

#### get\_block\_header

```python
def get_block_header(hash: BlockHash) -> Hex
```

Block header.

Returns the hex-encoded 80-byte block header.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-header)*

Endpoint: `GET /api/block/{hash}/header`

<a id="bitview_client.BitviewClient.get_block_hash_by_height"></a>

#### get\_block\_hash\_by\_height

```python
def get_block_hash_by_height(height: Height) -> BlockHash
```

Block hash by height.

Block hash as plain text.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-height)*

Endpoint: `GET /api/block-height/{height}`

<a id="bitview_client.BitviewClient.get_block_by_timestamp"></a>

#### get\_block\_by\_timestamp

```python
def get_block_by_timestamp(timestamp: Timestamp) -> BlockTimestamp
```

Block by timestamp.

Find the block with the greatest header timestamp at or before the given UNIX timestamp, choosing the earliest height on ties.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-timestamp)*

Endpoint: `GET /api/v1/mining/blocks/timestamp/{timestamp}`

<a id="bitview_client.BitviewClient.get_block_raw"></a>

#### get\_block\_raw

```python
def get_block_raw(hash: BlockHash) -> bytes
```

Raw block.

Returns the raw block data in binary format.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-raw)*

Endpoint: `GET /api/block/{hash}/raw`

<a id="bitview_client.BitviewClient.get_block_status"></a>

#### get\_block\_status

```python
def get_block_status(hash: BlockHash) -> BlockStatus
```

Block status.

Whether the block is in the best chain and, if so, its height and the hash of the next block. A block outside the best chain (orphaned or unknown) reports `in_best_chain: false`.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-status)*

Endpoint: `GET /api/block/{hash}/status`

<a id="bitview_client.BitviewClient.get_block_tip_height"></a>

#### get\_block\_tip\_height

```python
def get_block_tip_height() -> Height
```

Block tip height.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-tip-height)*

Endpoint: `GET /api/blocks/tip/height`

<a id="bitview_client.BitviewClient.get_block_tip_hash"></a>

#### get\_block\_tip\_hash

```python
def get_block_tip_hash() -> BlockHash
```

Block tip hash.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-tip-hash)*

Endpoint: `GET /api/blocks/tip/hash`

<a id="bitview_client.BitviewClient.get_block_txid"></a>

#### get\_block\_txid

```python
def get_block_txid(hash: BlockHash, index: BlockTxIndex) -> Txid
```

Transaction ID at index.

Retrieve a single transaction ID at a specific index within a block. Returns plain text txid.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transaction-id)*

Endpoint: `GET /api/block/{hash}/txid/{index}`

<a id="bitview_client.BitviewClient.get_block_txids"></a>

#### get\_block\_txids

```python
def get_block_txids(hash: BlockHash) -> List[Txid]
```

Block transaction IDs.

Transaction IDs in block order.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transaction-ids)*

Endpoint: `GET /api/block/{hash}/txids`

<a id="bitview_client.BitviewClient.get_block_txs"></a>

#### get\_block\_txs

```python
def get_block_txs(hash: BlockHash) -> List[Transaction]
```

Block transactions.

Up to 25 transactions starting at index 0.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transactions)*

Endpoint: `GET /api/block/{hash}/txs`

<a id="bitview_client.BitviewClient.get_block_txs_from_index"></a>

#### get\_block\_txs\_from\_index

```python
def get_block_txs_from_index(hash: BlockHash,
                             start_index: BlockTxIndex) -> List[Transaction]
```

Block transactions (paginated).

Up to 25 transactions starting at `start_index`, a multiple of 25.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transactions)*

Endpoint: `GET /api/block/{hash}/txs/{start_index}`

<a id="bitview_client.BitviewClient.get_blocks"></a>

#### get\_blocks

```python
def get_blocks() -> List[BlockInfo]
```

Recent blocks.

The last 10 blocks.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks)*

Endpoint: `GET /api/blocks`

<a id="bitview_client.BitviewClient.get_blocks_from_height"></a>

#### get\_blocks\_from\_height

```python
def get_blocks_from_height(height: Height) -> List[BlockInfo]
```

Blocks from height.

Retrieve up to 10 blocks going backwards from the given height. For example, height=100 returns blocks 100, 99, 98, ..., 91. Height=0 returns only block 0. A height past the tip is not found.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks)*

Endpoint: `GET /api/blocks/{height}`

<a id="bitview_client.BitviewClient.get_blocks_v1"></a>

#### get\_blocks\_v1

```python
def get_blocks_v1() -> List[BlockInfoV1]
```

Recent blocks with extras.

Retrieve the last 15 blocks with extended data including pool identification and fee statistics.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks-v1)*

Endpoint: `GET /api/v1/blocks`

<a id="bitview_client.BitviewClient.get_blocks_v1_from_height"></a>

#### get\_blocks\_v1\_from\_height

```python
def get_blocks_v1_from_height(height: Height) -> List[BlockInfoV1]
```

Blocks from height with extras.

Retrieve up to 15 blocks with extended data going backwards from the given height: the published blocks among the 15 heights ending at it, so fewer (or none) past the tip.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks-v1)*

Endpoint: `GET /api/v1/blocks/{height}`

<a id="bitview_client.BitviewClient.get_pools"></a>

#### get\_pools

```python
def get_pools() -> List[PoolInfo]
```

List all mining pools.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pools)*

Endpoint: `GET /api/v1/mining/pools`

<a id="bitview_client.BitviewClient.get_pool_ranking"></a>

#### get\_pool\_ranking

```python
def get_pool_ranking(time_period: TimePeriod) -> PoolsSummary
```

Mining pool statistics.

Get mining pool statistics for a time period.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pools)*

Endpoint: `GET /api/v1/mining/pools/{time_period}`

<a id="bitview_client.BitviewClient.get_pool"></a>

#### get\_pool

```python
def get_pool(slug: PoolSlug) -> PoolDetail
```

Mining pool details.

Block counts and mining shares by time period for one pool.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool)*

Endpoint: `GET /api/v1/mining/pool/{slug}`

<a id="bitview_client.BitviewClient.get_pools_hashrate"></a>

#### get\_pools\_hashrate

```python
def get_pools_hashrate() -> List[PoolHashrateEntry]
```

All pools hashrate (all time).

Get weekly hashrate data, over the whole chain, of the mining pools that mined in the last week, by time and then their weekly rank.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-hashrates)*

Endpoint: `GET /api/v1/mining/hashrate/pools`

<a id="bitview_client.BitviewClient.get_pools_hashrate_by_period"></a>

#### get\_pools\_hashrate\_by\_period

```python
def get_pools_hashrate_by_period(
        time_period: TimePeriod) -> List[PoolHashrateEntry]
```

All pools hashrate.

Get weekly hashrate data, for a time period, of the mining pools that mined in the last week, by time and then their weekly rank.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-hashrates)*

Endpoint: `GET /api/v1/mining/hashrate/pools/{time_period}`

<a id="bitview_client.BitviewClient.get_pool_hashrate"></a>

#### get\_pool\_hashrate

```python
def get_pool_hashrate(slug: PoolSlug) -> List[PoolHashrateEntry]
```

Mining pool hashrate.

Get hashrate history for a specific mining pool.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-hashrate)*

Endpoint: `GET /api/v1/mining/pool/{slug}/hashrate`

<a id="bitview_client.BitviewClient.get_pool_blocks"></a>

#### get\_pool\_blocks

```python
def get_pool_blocks(slug: PoolSlug) -> List[BlockInfoV1]
```

Mining pool blocks.

Get up to 100 recent blocks mined by a specific pool.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-blocks)*

Endpoint: `GET /api/v1/mining/pool/{slug}/blocks`

<a id="bitview_client.BitviewClient.get_pool_blocks_from_height"></a>

#### get\_pool\_blocks\_from\_height

```python
def get_pool_blocks_from_height(slug: PoolSlug,
                                height: Height) -> List[BlockInfoV1]
```

Mining pool blocks before height.

Get up to 100 blocks mined by a specific pool strictly below the given height (pass the last height of a page to get the next one).

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-blocks)*

Endpoint: `GET /api/v1/mining/pool/{slug}/blocks/{height}`

<a id="bitview_client.BitviewClient.get_hashrate"></a>

#### get\_hashrate

```python
def get_hashrate() -> HashrateSummary
```

Network hashrate (all time).

Get network hashrate and difficulty data for all time.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-hashrate)*

Endpoint: `GET /api/v1/mining/hashrate`

<a id="bitview_client.BitviewClient.get_hashrate_by_period"></a>

#### get\_hashrate\_by\_period

```python
def get_hashrate_by_period(time_period: TimePeriod) -> HashrateSummary
```

Network hashrate.

Get network hashrate and difficulty data for a time period.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-hashrate)*

Endpoint: `GET /api/v1/mining/hashrate/{time_period}`

<a id="bitview_client.BitviewClient.get_difficulty_adjustments"></a>

#### get\_difficulty\_adjustments

```python
def get_difficulty_adjustments() -> List[DifficultyAdjustmentEntry]
```

Difficulty adjustments (all time).

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustments)*

Endpoint: `GET /api/v1/mining/difficulty-adjustments`

<a id="bitview_client.BitviewClient.get_difficulty_adjustments_by_period"></a>

#### get\_difficulty\_adjustments\_by\_period

```python
def get_difficulty_adjustments_by_period(
        time_period: TimePeriod) -> List[DifficultyAdjustmentEntry]
```

Difficulty adjustments.

Get historical difficulty adjustments for a time period.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustments)*

Endpoint: `GET /api/v1/mining/difficulty-adjustments/{time_period}`

<a id="bitview_client.BitviewClient.get_reward_stats"></a>

#### get\_reward\_stats

```python
def get_reward_stats(block_count: int) -> RewardStats
```

Mining reward statistics.

Get mining reward statistics for the last N blocks including total rewards, fees, and transaction count.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-reward-stats)*

Endpoint: `GET /api/v1/mining/reward-stats/{block_count}`

<a id="bitview_client.BitviewClient.get_block_fees"></a>

#### get\_block\_fees

```python
def get_block_fees(time_period: TimePeriod) -> List[BlockFeesEntry]
```

Block fees.

Get average total fees per block for a time period.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-fees)*

Endpoint: `GET /api/v1/mining/blocks/fees/{time_period}`

<a id="bitview_client.BitviewClient.get_block_rewards"></a>

#### get\_block\_rewards

```python
def get_block_rewards(time_period: TimePeriod) -> List[BlockRewardsEntry]
```

Block rewards.

Get average coinbase reward (subsidy + fees) per block for a time period.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-rewards)*

Endpoint: `GET /api/v1/mining/blocks/rewards/{time_period}`

<a id="bitview_client.BitviewClient.get_block_fee_rates"></a>

#### get\_block\_fee\_rates

```python
def get_block_fee_rates(time_period: TimePeriod) -> List[BlockFeeRatesEntry]
```

Block fee rates.

Get block fee rate percentiles (min, 10th, 25th, median, 75th, 90th, max) for a time period, each a whole sat/vB (`/precise` for 3 decimals).

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-feerates)*

Endpoint: `GET /api/v1/mining/blocks/fee-rates/{time_period}`

<a id="bitview_client.BitviewClient.get_block_fee_rates_precise"></a>

#### get\_block\_fee\_rates\_precise

```python
def get_block_fee_rates_precise(
        time_period: TimePeriod) -> List[BlockFeeRatesPreciseEntry]
```

Block fee rates (precise).

Get block fee rate percentiles (min, 10th, 25th, median, 75th, 90th, max) for a time period, each in sat/vB to 3 decimals.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-feerates)*

Endpoint: `GET /api/v1/mining/blocks/fee-rates/{time_period}/precise`

<a id="bitview_client.BitviewClient.get_block_sizes_weights"></a>

#### get\_block\_sizes\_weights

```python
def get_block_sizes_weights(time_period: TimePeriod) -> BlockSizesWeights
```

Block sizes and weights.

Get average block sizes and weights for a time period.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-sizes-weights)*

Endpoint: `GET /api/v1/mining/blocks/sizes-weights/{time_period}`

<a id="bitview_client.BitviewClient.get_mempool_blocks"></a>

#### get\_mempool\_blocks

```python
def get_mempool_blocks() -> List[MempoolBlock]
```

Projected mempool blocks.

Projected blocks for fee estimation. Block 0 reflects Bitcoin Core's actual next-block selection; blocks 1+ are a fee-tier approximation.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool-blocks-fees)*

Endpoint: `GET /api/v1/fees/mempool-blocks`

<a id="bitview_client.BitviewClient.get_recommended_fees"></a>

#### get\_recommended\_fees

```python
def get_recommended_fees() -> RecommendedFees
```

Recommended fees.

Recommended fee rates by confirmation target, in whole sat/vB.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-recommended-fees)*

Endpoint: `GET /api/v1/fees/recommended`

<a id="bitview_client.BitviewClient.get_precise_fees"></a>

#### get\_precise\_fees

```python
def get_precise_fees() -> PreciseRecommendedFees
```

Recommended fee rates (precise).

Recommended fee rates by confirmation target, with up to three decimal places and support for sub-sat/vB rates.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-recommended-fees-precise)*

Endpoint: `GET /api/v1/fees/precise`

<a id="bitview_client.BitviewClient.get_mempool"></a>

#### get\_mempool

```python
def get_mempool() -> MempoolInfo
```

Mempool statistics.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool)*

Endpoint: `GET /api/mempool`

<a id="bitview_client.BitviewClient.get_mempool_txids"></a>

#### get\_mempool\_txids

```python
def get_mempool_txids() -> List[Txid]
```

Mempool transaction IDs.

Get all transaction IDs currently in the mempool.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool-transaction-ids)*

Endpoint: `GET /api/mempool/txids`

<a id="bitview_client.BitviewClient.get_mempool_recent"></a>

#### get\_mempool\_recent

```python
def get_mempool_recent() -> List[MempoolRecentTx]
```

Recent mempool transactions.

Get the last 10 transactions to enter the mempool.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool-recent)*

Endpoint: `GET /api/mempool/recent`

<a id="bitview_client.BitviewClient.get_replacements"></a>

#### get\_replacements

```python
def get_replacements() -> List[ReplacementNode]
```

Recent RBF replacements.

Returns up to 25 most-recent RBF replacement trees across the whole mempool. Each entry has the shape of `replacements` in `GET /api/v1/tx/{txid}/rbf`.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-replacements)*

Endpoint: `GET /api/v1/replacements`

<a id="bitview_client.BitviewClient.get_fullrbf_replacements"></a>

#### get\_fullrbf\_replacements

```python
def get_fullrbf_replacements() -> List[ReplacementNode]
```

Recent full-RBF replacements.

Same response shape as `GET /api/v1/replacements`, but limited to trees where at least one predecessor was non-signaling (full-RBF).

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-fullrbf-replacements)*

Endpoint: `GET /api/v1/fullrbf/replacements`

<a id="bitview_client.BitviewClient.get_mempool_block"></a>

#### get\_mempool\_block

```python
def get_mempool_block(index: MempoolBlockIndex) -> MempoolBlockContent
```

Projected mempool block.

A projected mempool block with its transactions in order and its stats, by position as `GET /api/v1/fees/mempool-blocks` lists them: 0 is the next block, Bitcoin Core's `getblocktemplate` selection, and the only one served for now (others answer 404). The returned `hash` is an opaque content token: poll `GET /api/mempool-blocks/{index}/diff/{hash}` to follow changes instead of refetching the block.

Endpoint: `GET /api/mempool-blocks/{index}`

<a id="bitview_client.BitviewClient.get_mempool_block_diff"></a>

#### get\_mempool\_block\_diff

```python
def get_mempool_block_diff(index: MempoolBlockIndex,
                           since: MempoolBlockHash) -> MempoolBlockDiff
```

Projected mempool block changes.

The projected mempool block's changes since the client's copy at `since`. `order` is the new block in order: each entry is either `[start, count]`, the client's transactions `start..start + count` kept in order, or a transaction object, a body to place there. Walk `order` once to rebuild, and replace the cached `stats` with the response's; `removed` lists the txids that left, so clients can evict cached bodies. Use the response `hash` as the next `since`; with `If-None-Match` an unchanged block answers 304. Returns `404` when `since` has aged out of server history: fetch `GET /api/mempool-blocks/{index}` again.

Endpoint: `GET /api/mempool-blocks/{index}/diff/{since}`

<a id="bitview_client.BitviewClient.get_txid_by_index"></a>

#### get\_txid\_by\_index

```python
def get_txid_by_index(index: TxIndex) -> Txid
```

Txid by index.

Retrieve the transaction ID (txid) at a given global transaction index. Returns the txid as plain text.

Endpoint: `GET /api/tx-index/{index}`

<a id="bitview_client.BitviewClient.get_cpfp"></a>

#### get\_cpfp

```python
def get_cpfp(txid: Txid) -> Cpfp
```

CPFP info.

Returns ancestors and descendants for a CPFP (Child Pays For Parent) transaction, including the effective fee rate of the package. A mempool transaction answers its live package; a confirmed one its package among its block's transactions; an unknown transaction, or a confirmed one without relatives in its block, `{"ancestors":[]}`.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-children-pay-for-parent)*

Endpoint: `GET /api/v1/cpfp/{txid}`

<a id="bitview_client.BitviewClient.get_tx_rbf"></a>

#### get\_tx\_rbf

```python
def get_tx_rbf(txid: Txid) -> RbfResponse
```

RBF replacement history.

Returns the RBF replacement tree for a transaction, if any. Both `replacements` and `replaces` are null when the tx has no known RBF history within the mempool monitor's retention window.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-rbf-history)*

Endpoint: `GET /api/v1/tx/{txid}/rbf`

<a id="bitview_client.BitviewClient.get_tx"></a>

#### get\_tx

```python
def get_tx(txid: Txid) -> Transaction
```

Transaction information.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction)*

Endpoint: `GET /api/tx/{txid}`

<a id="bitview_client.BitviewClient.get_tx_hex"></a>

#### get\_tx\_hex

```python
def get_tx_hex(txid: Txid) -> Hex
```

Transaction hex.

Hex-encoded serialized transaction.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-hex)*

Endpoint: `GET /api/tx/{txid}/hex`

<a id="bitview_client.BitviewClient.get_tx_merkleblock_proof"></a>

#### get\_tx\_merkleblock\_proof

```python
def get_tx_merkleblock_proof(txid: Txid) -> Hex
```

Transaction merkleblock proof.

Get the merkleblock proof for a transaction (BIP37 format, hex encoded).

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-merkleblock-proof)*

Endpoint: `GET /api/tx/{txid}/merkleblock-proof`

<a id="bitview_client.BitviewClient.get_tx_merkle_proof"></a>

#### get\_tx\_merkle\_proof

```python
def get_tx_merkle_proof(txid: Txid) -> MerkleProof
```

Transaction merkle proof.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-merkle-proof)*

Endpoint: `GET /api/tx/{txid}/merkle-proof`

<a id="bitview_client.BitviewClient.get_tx_outspend"></a>

#### get\_tx\_outspend

```python
def get_tx_outspend(txid: Txid, vout: Vout) -> TxOutspend
```

Output spend status.

Whether the output has been spent and, if so, the spending transaction details. An output no known transaction holds is unspent.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-outspend)*

Endpoint: `GET /api/tx/{txid}/outspend/{vout}`

<a id="bitview_client.BitviewClient.get_tx_outspends"></a>

#### get\_tx\_outspends

```python
def get_tx_outspends(txid: Txid) -> List[TxOutspend]
```

All output spend statuses.

Spend statuses in output-index order.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-outspends)*

Endpoint: `GET /api/tx/{txid}/outspends`

<a id="bitview_client.BitviewClient.get_tx_raw"></a>

#### get\_tx\_raw

```python
def get_tx_raw(txid: Txid) -> bytes
```

Transaction raw.

Returns a transaction as binary data.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-raw)*

Endpoint: `GET /api/tx/{txid}/raw`

<a id="bitview_client.BitviewClient.get_tx_status"></a>

#### get\_tx\_status

```python
def get_tx_status(txid: Txid) -> TxStatus
```

Transaction status.

Retrieve the confirmation status of a transaction. Returns whether the transaction is confirmed and, if so, the block height, hash, and timestamp. A transaction not in the chain, pending or unknown, is `{"confirmed":false}`.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-status)*

Endpoint: `GET /api/tx/{txid}/status`

<a id="bitview_client.BitviewClient.get_transaction_times"></a>

#### get\_transaction\_times

```python
def get_transaction_times(txids: List[Txid]) -> List[Timestamp]
```

Transaction first-seen times.

First-seen UNIX timestamps in seconds, in request order, of transactions in the mempool or that recently left it other than by replacement (mined ones included); 0 otherwise.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-times)*

Endpoint: `GET /api/v1/transaction-times`

<a id="bitview_client.BitviewClient.post_tx"></a>

#### post\_tx

```python
def post_tx(body: str) -> Txid
```

Broadcast transaction.

Submit a raw transaction as hexadecimal text (at most 8,000,000 request bytes, including whitespace). Returns its txid as plain text. No responses are cached. Cancellation or a transport error after dispatch may leave the submission outcome unknown; do not automatically retry.

*[Mempool.space docs](https://mempool.space/docs/api/rest#post-transaction)*

Endpoint: `POST /api/tx`

<a id="bitview_client.BitviewClient.get_utxo_set_latest"></a>

#### get\_utxo\_set\_latest

```python
def get_utxo_set_latest() -> UtxoSet
```

Latest UTXO set.

The UTXO set after the latest published block, in the layout of `GET /api/utxo-set/{point}`.

Endpoint: `GET /api/utxo-set`

<a id="bitview_client.BitviewClient.get_utxo_set"></a>

#### get\_utxo\_set

```python
def get_utxo_set(point: HeightOrDate) -> UtxoSet
```

UTXO set at block height or date.

The UTXO set after a block (`840000`) or after the last published block of a UTC day (`YYYY-MM-DD`), by creation height. Returns `{ height, hash, date, count, supply, origins }`; entry `i` of `origins.count` and `origins.supply` (BTC) is what remains unspent of block `i`'s outputs.

Endpoint: `GET /api/utxo-set/{point}`

<a id="bitview_client.BitviewClient.get_utxo_set_diff"></a>

#### get\_utxo\_set\_diff

```python
def get_utxo_set_diff(point: HeightOrDate) -> UtxoSetDiff
```

UTXO set changes of a block or date.

What a block (`840000`) or a UTC day's published blocks (`YYYY-MM-DD`) changed in the UTXO set. Returns `{ first, last, hash, date, created, spent }`: `created` has one row per block, `spent` one row per creation height, each as columnar `height`, `count` and `supply` (BTC). The set after block `first - 1` plus `created` minus `spent` is the set after block `last`.

Endpoint: `GET /api/utxo-set/{point}/diff`

<a id="bitview_client.BitviewClient.get_live_price"></a>

#### get\_live\_price

```python
def get_live_price() -> Dollars
```

Live BTC/USD price.

Current BTC/USD price in dollars, derived from on-chain round-dollar output patterns in the oracle's recent-block window, with the mempool's outputs blended in when available. Confirmed per-height history is available at `GET /api/series/price/height`.

Endpoint: `GET /api/oracle/price`

<a id="bitview_client.BitviewClient.get_oracle_histogram_payments_live"></a>

#### get\_oracle\_histogram\_payments\_live

```python
def get_oracle_histogram_payments_live() -> List[int]
```

Live payment output histogram.

Live smoothed histogram of oracle-eligible payment outputs, binned by output value on the oracle log scale. It combines the committed oracle window with the complete mempool's eligible outputs from a matching chain publication. A flat array of log-scale bins.

Endpoint: `GET /api/oracle/histogram/payments/live`

<a id="bitview_client.BitviewClient.get_oracle_histogram_payments"></a>

#### get\_oracle\_histogram\_payments

```python
def get_oracle_histogram_payments(point: HeightOrDate) -> List[int]
```

Payment output histogram at height or day.

Smoothed histogram of oracle-eligible payment outputs for a confirmed point. A block height (`840000`) gives that block's oracle payment histogram; a calendar date (`YYYY-MM-DD`) gives the average of that day's per-block payment histograms. A flat array of log-scale bins.

Endpoint: `GET /api/oracle/histogram/payments/{point}`

<a id="bitview_client.BitviewClient.get_oracle_histogram_outputs_live"></a>

#### get\_oracle\_histogram\_outputs\_live

```python
def get_oracle_histogram_outputs_live() -> List[int]
```

Live output value histogram.

Live unfiltered output value histogram for the complete published mempool. Nonzero outputs within the histogram's bin range are counted by value on the oracle log scale; no oracle payment filters are applied. A flat array of log-scale bins, all zero when no mempool is configured.

Endpoint: `GET /api/oracle/histogram/outputs/live`

<a id="bitview_client.BitviewClient.get_oracle_histogram_outputs"></a>

#### get\_oracle\_histogram\_outputs

```python
def get_oracle_histogram_outputs(point: HeightOrDate) -> List[int]
```

Output value histogram at height or day.

Unfiltered output value histogram for a confirmed point. A block height (`840000`) counts that block's nonzero outputs within the histogram's bin range, coinbase included, by value on the oracle log scale; a calendar date (`YYYY-MM-DD`) sums every block that day. A flat array of log-scale bins.

Endpoint: `GET /api/oracle/histogram/outputs/{point}`

<a id="bitview_client.BitviewClient.get_openapi"></a>

#### get\_openapi

```python
def get_openapi() -> Any
```

OpenAPI specification.

Full OpenAPI 3.1 specification for this API. Build tools from this document.

Endpoint: `GET /openapi.json`

<a id="bitview_client.BitviewClient.get_api"></a>

#### get\_api

```python
def get_api() -> Any
```

Compact API summary.

Compact custom summary of the OpenAPI specification for LLM reading: types instead of schemas, `returns` instead of responses, no examples; parameters keep their enum, default and bounds. Not valid OpenAPI: build tools from `GET /openapi.json`.

Endpoint: `GET /api.json`

