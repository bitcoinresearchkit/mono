# Table of Contents

* [brk\_client](#bitview_client)
  * [BitviewError](#bitview_client.BitviewError)
  * [BitviewClient](#bitview_client.BitviewClient)
    * [VERSION](#bitview_client.BitviewClient.VERSION)
    * [INDEXES](#bitview_client.BitviewClient.INDEXES)
    * [POOL\_ID\_TO\_POOL\_NAME](#bitview_client.BitviewClient.POOL_ID_TO_POOL_NAME)
    * [TERM\_NAMES](#bitview_client.BitviewClient.TERM_NAMES)
    * [EPOCH\_NAMES](#bitview_client.BitviewClient.EPOCH_NAMES)
    * [CLASS\_NAMES](#bitview_client.BitviewClient.CLASS_NAMES)
    * [ENTRY\_NAMES](#bitview_client.BitviewClient.ENTRY_NAMES)
    * [SPENDABLE\_TYPE\_NAMES](#bitview_client.BitviewClient.SPENDABLE_TYPE_NAMES)
    * [AGE\_RANGE\_NAMES](#bitview_client.BitviewClient.AGE_RANGE_NAMES)
    * [UNDER\_AGE\_NAMES](#bitview_client.BitviewClient.UNDER_AGE_NAMES)
    * [OVER\_AGE\_NAMES](#bitview_client.BitviewClient.OVER_AGE_NAMES)
    * [AMOUNT\_RANGE\_NAMES](#bitview_client.BitviewClient.AMOUNT_RANGE_NAMES)
    * [OVER\_AMOUNT\_NAMES](#bitview_client.BitviewClient.OVER_AMOUNT_NAMES)
    * [UNDER\_AMOUNT\_NAMES](#bitview_client.BitviewClient.UNDER_AMOUNT_NAMES)
    * [PROFITABILITY\_RANGE\_NAMES](#bitview_client.BitviewClient.PROFITABILITY_RANGE_NAMES)
    * [PROFIT\_NAMES](#bitview_client.BitviewClient.PROFIT_NAMES)
    * [LOSS\_NAMES](#bitview_client.BitviewClient.LOSS_NAMES)
    * [\_\_init\_\_](#bitview_client.BitviewClient.__init__)
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
    * [get\_indexes](#bitview_client.BitviewClient.get_indexes)
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
    * [list\_urpd\_dates](#bitview_client.BitviewClient.list_urpd_dates)
    * [get\_urpd](#bitview_client.BitviewClient.get_urpd)
    * [get\_urpd\_at](#bitview_client.BitviewClient.get_urpd_at)
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
    * [get\_block\_by\_height](#bitview_client.BitviewClient.get_block_by_height)
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
    * [get\_pool\_stats](#bitview_client.BitviewClient.get_pool_stats)
    * [get\_pool](#bitview_client.BitviewClient.get_pool)
    * [get\_pools\_hashrate](#bitview_client.BitviewClient.get_pools_hashrate)
    * [get\_pools\_hashrate\_by\_period](#bitview_client.BitviewClient.get_pools_hashrate_by_period)
    * [get\_pool\_hashrate](#bitview_client.BitviewClient.get_pool_hashrate)
    * [get\_pool\_blocks](#bitview_client.BitviewClient.get_pool_blocks)
    * [get\_pool\_blocks\_from](#bitview_client.BitviewClient.get_pool_blocks_from)
    * [get\_hashrate](#bitview_client.BitviewClient.get_hashrate)
    * [get\_hashrate\_by\_period](#bitview_client.BitviewClient.get_hashrate_by_period)
    * [get\_difficulty\_adjustments](#bitview_client.BitviewClient.get_difficulty_adjustments)
    * [get\_difficulty\_adjustments\_by\_period](#bitview_client.BitviewClient.get_difficulty_adjustments_by_period)
    * [get\_reward\_stats](#bitview_client.BitviewClient.get_reward_stats)
    * [get\_block\_fees](#bitview_client.BitviewClient.get_block_fees)
    * [get\_block\_rewards](#bitview_client.BitviewClient.get_block_rewards)
    * [get\_block\_fee\_rates](#bitview_client.BitviewClient.get_block_fee_rates)
    * [get\_block\_sizes\_weights](#bitview_client.BitviewClient.get_block_sizes_weights)
    * [get\_mempool\_blocks](#bitview_client.BitviewClient.get_mempool_blocks)
    * [get\_recommended\_fees](#bitview_client.BitviewClient.get_recommended_fees)
    * [get\_precise\_fees](#bitview_client.BitviewClient.get_precise_fees)
    * [get\_mempool](#bitview_client.BitviewClient.get_mempool)
    * [get\_mempool\_hash](#bitview_client.BitviewClient.get_mempool_hash)
    * [get\_mempool\_txids](#bitview_client.BitviewClient.get_mempool_txids)
    * [get\_mempool\_recent](#bitview_client.BitviewClient.get_mempool_recent)
    * [get\_replacements](#bitview_client.BitviewClient.get_replacements)
    * [get\_fullrbf\_replacements](#bitview_client.BitviewClient.get_fullrbf_replacements)
    * [get\_block\_template](#bitview_client.BitviewClient.get_block_template)
    * [get\_block\_template\_diff](#bitview_client.BitviewClient.get_block_template_diff)
    * [get\_live\_price](#bitview_client.BitviewClient.get_live_price)
    * [get\_oracle\_price](#bitview_client.BitviewClient.get_oracle_price)
    * [get\_oracle\_histogram\_payments\_live](#bitview_client.BitviewClient.get_oracle_histogram_payments_live)
    * [get\_oracle\_histogram\_payments](#bitview_client.BitviewClient.get_oracle_histogram_payments)
    * [get\_oracle\_histogram\_outputs\_live](#bitview_client.BitviewClient.get_oracle_histogram_outputs_live)
    * [get\_oracle\_histogram\_outputs](#bitview_client.BitviewClient.get_oracle_histogram_outputs)
    * [get\_tx\_by\_index](#bitview_client.BitviewClient.get_tx_by_index)
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
    * [get\_openapi](#bitview_client.BitviewClient.get_openapi)
    * [get\_api](#bitview_client.BitviewClient.get_api)

<a id="bitview_client"></a>

# brk\_client

<a id="bitview_client.BitviewError"></a>

## BitviewError Objects

```python
class BitviewError(Exception)
```

Custom error class for Bitview client errors.

<a id="bitview_client.BitviewClient"></a>

## BitviewClient Objects

```python
class BitviewClient(BitviewClientBase)
```

Main Bitview client with series tree and API methods.

<a id="bitview_client.BitviewClient.VERSION"></a>

#### VERSION

<a id="bitview_client.BitviewClient.INDEXES"></a>

#### INDEXES

<a id="bitview_client.BitviewClient.POOL_ID_TO_POOL_NAME"></a>

#### POOL\_ID\_TO\_POOL\_NAME

<a id="bitview_client.BitviewClient.TERM_NAMES"></a>

#### TERM\_NAMES

<a id="bitview_client.BitviewClient.EPOCH_NAMES"></a>

#### EPOCH\_NAMES

<a id="bitview_client.BitviewClient.CLASS_NAMES"></a>

#### CLASS\_NAMES

<a id="bitview_client.BitviewClient.ENTRY_NAMES"></a>

#### ENTRY\_NAMES

<a id="bitview_client.BitviewClient.SPENDABLE_TYPE_NAMES"></a>

#### SPENDABLE\_TYPE\_NAMES

<a id="bitview_client.BitviewClient.AGE_RANGE_NAMES"></a>

#### AGE\_RANGE\_NAMES

<a id="bitview_client.BitviewClient.UNDER_AGE_NAMES"></a>

#### UNDER\_AGE\_NAMES

<a id="bitview_client.BitviewClient.OVER_AGE_NAMES"></a>

#### OVER\_AGE\_NAMES

<a id="bitview_client.BitviewClient.AMOUNT_RANGE_NAMES"></a>

#### AMOUNT\_RANGE\_NAMES

<a id="bitview_client.BitviewClient.OVER_AMOUNT_NAMES"></a>

#### OVER\_AMOUNT\_NAMES

<a id="bitview_client.BitviewClient.UNDER_AMOUNT_NAMES"></a>

#### UNDER\_AMOUNT\_NAMES

<a id="bitview_client.BitviewClient.PROFITABILITY_RANGE_NAMES"></a>

#### PROFITABILITY\_RANGE\_NAMES

<a id="bitview_client.BitviewClient.PROFIT_NAMES"></a>

#### PROFIT\_NAMES

<a id="bitview_client.BitviewClient.LOSS_NAMES"></a>

#### LOSS\_NAMES

<a id="bitview_client.BitviewClient.__init__"></a>

#### \_\_init\_\_

```python
def __init__(base_url: str = 'http://localhost:3000', timeout: float = 30.0)
```

<a id="bitview_client.BitviewClient.series_endpoint"></a>

#### series\_endpoint

```python
def series_endpoint(series: str, index: Index) -> SeriesEndpoint[Any]
```

Create a dynamic series endpoint builder for any series/index combination.

Use this for programmatic access when the series name is determined at runtime.
For type-safe access, use the `series` tree instead.

<a id="bitview_client.BitviewClient.index_to_date"></a>

#### index\_to\_date

```python
def index_to_date(index: Index, i: int) -> Union[date, datetime]
```

Convert an index value to a date/datetime for date-based indexes.

<a id="bitview_client.BitviewClient.date_to_index"></a>

#### date\_to\_index

```python
def date_to_index(index: Index, d: Union[date, datetime]) -> int
```

Convert a date/datetime to an index value for date-based indexes.

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
        addr_type: OutputType, payload: Union[bytes, bytearray, memoryview],
        nibbles: int) -> AddrHashPrefixMatches
```

Fetch address hash-prefix matches from raw payload bytes matching addr_type length.

<a id="bitview_client.BitviewClient.get_health"></a>

#### get\_health

```python
def get_health() -> Health
```

Health check.

Liveness probe. Returns server identity, uptime, and indexed/computed heights from local state only (no bitcoind round-trip). For real chain-tip catch-up, request `GET /api/server/sync`.

Endpoint: `GET /health`

<a id="bitview_client.BitviewClient.get_version"></a>

#### get\_version

```python
def get_version() -> str
```

API version.

Returns the current version of the API server

Endpoint: `GET /version`

<a id="bitview_client.BitviewClient.get_sync_status"></a>

#### get\_sync\_status

```python
def get_sync_status() -> SyncStatus
```

Sync status.

Returns the sync status of the indexer, including indexed height, tip height, blocks behind, and last indexed timestamp.

Endpoint: `GET /api/server/sync`

<a id="bitview_client.BitviewClient.get_disk_usage"></a>

#### get\_disk\_usage

```python
def get_disk_usage() -> DiskUsage
```

Disk usage.

Returns the disk space used by BRK and Bitcoin data.

Endpoint: `GET /api/server/disk`

<a id="bitview_client.BitviewClient.get_series_tree"></a>

#### get\_series\_tree

```python
def get_series_tree() -> TreeNode
```

Series catalog.

Returns the complete hierarchical catalog of available series organized as a tree structure. Series are grouped by categories and subcategories.

Endpoint: `GET /api/series`

<a id="bitview_client.BitviewClient.get_series_count"></a>

#### get\_series\_count

```python
def get_series_count() -> DetailedSeriesCount
```

Series count.

Returns the number of series available per index type.

Endpoint: `GET /api/series/count`

<a id="bitview_client.BitviewClient.get_indexes"></a>

#### get\_indexes

```python
def get_indexes() -> List[IndexInfo]
```

List available indexes.

Returns all available indexes with their accepted query aliases. Use any alias when querying series.

Endpoint: `GET /api/series/indexes`

<a id="bitview_client.BitviewClient.list_series"></a>

#### list\_series

```python
def list_series(page: Optional[int] = None,
                per_page: Optional[int] = None) -> PaginatedSeries
```

Series list.

Paginated flat list of all available series names. Use `page` query param for pagination.

Endpoint: `GET /api/series/list`

<a id="bitview_client.BitviewClient.search_series"></a>

#### search\_series

```python
def search_series(q: SeriesName, limit: Optional[Limit] = None) -> List[str]
```

Search series.

Search series by name or descriptive terms. Matches metric names, descriptions, formulas, cohort aliases, partial words, and common typos.

Endpoint: `GET /api/series/search`

<a id="bitview_client.BitviewClient.get_series_info"></a>

#### get\_series\_info

```python
def get_series_info(series: SeriesName) -> SeriesInfo
```

Get series info.

Returns the optional description, supported indexes, and value type for the specified series.

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

Fetch data for a specific series at the given index. Use query parameters to filter by date range and format (json/csv).

Endpoint: `GET /api/series/{series}/{index}`

<a id="bitview_client.BitviewClient.get_series_data"></a>

#### get\_series\_data

```python
def get_series_data(series: SeriesName,
                    index: Index,
                    start: Optional[RangeIndex] = None,
                    end: Optional[RangeIndex] = None,
                    limit: Optional[Limit] = None,
                    format: Optional[Format] = None) -> Union[List[bool], str]
```

Get raw series data.

Returns just the data array without the SeriesData wrapper. Supports the same range and format parameters as `GET /api/series/{series}/{index}`.

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

Get series data length.

Returns the total number of data points for a series at the given index.

Endpoint: `GET /api/series/{series}/{index}/len`

<a id="bitview_client.BitviewClient.get_series_version"></a>

#### get\_series\_version

```python
def get_series_version(series: SeriesName, index: Index) -> Version
```

Get series version.

Returns the current version of a series. Changes when the series data is updated.

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
def list_urpd_cohorts() -> List[Cohort]
```

Available URPD cohorts.

Cohorts for which URPD data is available. Returns names like `all`, `sth`, `lth`, `utxos_under_1h_old`.

Endpoint: `GET /api/urpd`

<a id="bitview_client.BitviewClient.list_urpd_dates"></a>

#### list\_urpd\_dates

```python
def list_urpd_dates(cohort: Cohort,
                    weight: Optional[UrpdWeight] = None) -> List[Date]
```

Available URPD dates.

Dates for which a URPD snapshot is available for the cohort and selected `weight`. One entry per UTC day, sorted ascending.

Endpoint: `GET /api/urpd/{cohort}/dates`

<a id="bitview_client.BitviewClient.get_urpd"></a>

#### get\_urpd

```python
def get_urpd(cohort: Cohort,
             agg: Optional[UrpdAggregation] = None,
             weight: Optional[UrpdWeight] = None) -> Urpd
```

Latest URPD.

URPD for the most recent available date in the cohort. The response's `date` field echoes which date was served. Returns `{ cohort, date, weight, aggregation, close, total_supply, buckets }`. `close` and each bucket's `price_floor`, `realized_cap`, and `unrealized_pnl` are USD; `total_supply` and bucket `supply` are BTC. `unrealized_pnl` can be negative.

Endpoint: `GET /api/urpd/{cohort}`

<a id="bitview_client.BitviewClient.get_urpd_at"></a>

#### get\_urpd\_at

```python
def get_urpd_at(cohort: Cohort,
                date: str,
                agg: Optional[UrpdAggregation] = None,
                weight: Optional[UrpdWeight] = None) -> Urpd
```

URPD at date.

URPD for a (cohort, date) pair. Returns `{ cohort, date, weight, aggregation, close, total_supply, buckets }` where each bucket is `{ price_floor, supply, realized_cap, unrealized_pnl }`. `close`, `price_floor`, `realized_cap`, and `unrealized_pnl` are USD; `total_supply` and `supply` are BTC. `unrealized_pnl` can be negative.

Endpoint: `GET /api/urpd/{cohort}/{date}`

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
def get_prices() -> Prices
```

Current BTC price.

Returns bitcoin latest price (on-chain derived, USD only).

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-price)*

Endpoint: `GET /api/v1/prices`

<a id="bitview_client.BitviewClient.get_historical_price"></a>

#### get\_historical\_price

```python
def get_historical_price(
        timestamp: Optional[Timestamp] = None) -> HistoricalPrice
```

Historical price.

Get historical BTC/USD price. Optionally specify a UNIX timestamp to get the price at that time.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-historical-price)*

Endpoint: `GET /api/v1/historical-price`

<a id="bitview_client.BitviewClient.get_address_hash_prefix_matches"></a>

#### get\_address\_hash\_prefix\_matches

```python
def get_address_hash_prefix_matches(addr_type: OutputType,
                                    prefix: str) -> AddrHashPrefixMatches
```

Address hash-prefix matches.

Find addresses by address type and by the first 1-16 hex nibbles of RapidHash v3 over the raw address payload bytes. Intended for privacy-preserving client-side wallet discovery without sending raw addresses or xpubs. Fetch metadata with `GET /api/address/{address}`.

Endpoint: `GET /api/address/hash-prefix/{addr_type}/{prefix}`

<a id="bitview_client.BitviewClient.get_address"></a>

#### get\_address

```python
def get_address(address: Addr) -> AddrStats
```

Address information.

Retrieve address information including current balance and transaction counts. Supports all standard Bitcoin address types (P2PKH, P2SH, P2WPKH, P2WSH, P2TR).

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address)*

Endpoint: `GET /api/address/{address}`

<a id="bitview_client.BitviewClient.get_address_txs"></a>

#### get\_address\_txs

```python
def get_address_txs(address: Addr) -> List[Transaction]
```

Address transactions.

Get transaction history for an address, newest first. Returns up to 50 mempool transactions plus a confirmed page sized to fill the response to 50 total (chain floor of 25, so 25-50 confirmed depending on mempool weight). To paginate further confirmed history, request `GET /api/address/{address}/txs/chain/{after_txid}` with the last returned txid.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions)*

Endpoint: `GET /api/address/{address}/txs`

<a id="bitview_client.BitviewClient.get_address_confirmed_txs"></a>

#### get\_address\_confirmed\_txs

```python
def get_address_confirmed_txs(address: Addr) -> List[Transaction]
```

Address confirmed transactions.

Get the first 25 confirmed transactions for an address. For pagination, request `GET /api/address/{address}/txs/chain/{after_txid}` with the last returned txid.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-chain)*

Endpoint: `GET /api/address/{address}/txs/chain`

<a id="bitview_client.BitviewClient.get_address_confirmed_txs_after"></a>

#### get\_address\_confirmed\_txs\_after

```python
def get_address_confirmed_txs_after(address: Addr,
                                    after_txid: Txid) -> List[Transaction]
```

Address confirmed transactions (paginated).

Get the next 25 confirmed transactions strictly older than `after_txid` (Esplora-canonical pagination form, matches mempool.space).

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-chain)*

Endpoint: `GET /api/address/{address}/txs/chain/{after_txid}`

<a id="bitview_client.BitviewClient.get_address_mempool_txs"></a>

#### get\_address\_mempool\_txs

```python
def get_address_mempool_txs(address: Addr) -> List[Transaction]
```

Address mempool transactions.

Get unconfirmed transactions for an address from the mempool, newest first (up to 50).

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-mempool)*

Endpoint: `GET /api/address/{address}/txs/mempool`

<a id="bitview_client.BitviewClient.get_address_utxos"></a>

#### get\_address\_utxos

```python
def get_address_utxos(address: Addr) -> List[Utxo]
```

Address UTXOs.

Get unspent transaction outputs (UTXOs) for an address. Returns txid, vout, value, and confirmation status for each UTXO.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-utxo)*

Endpoint: `GET /api/address/{address}/utxo`

<a id="bitview_client.BitviewClient.validate_address"></a>

#### validate\_address

```python
def validate_address(address: str) -> AddrValidation
```

Validate address.

Validate a Bitcoin address and get information about its type and scriptPubKey. Returns `isvalid: false` with an error message for invalid addresses.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-validate)*

Endpoint: `GET /api/v1/validate-address/{address}`

<a id="bitview_client.BitviewClient.get_block"></a>

#### get\_block

```python
def get_block(hash: BlockHash) -> BlockInfo
```

Block information.

Retrieve block information by block hash. Returns block metadata including height, timestamp, difficulty, size, weight, and transaction count.

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

<a id="bitview_client.BitviewClient.get_block_by_height"></a>

#### get\_block\_by\_height

```python
def get_block_by_height(height: Height) -> BlockHash
```

Block hash by height.

Retrieve the block hash at a given height. Returns the hash as plain text.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-height)*

Endpoint: `GET /api/block-height/{height}`

<a id="bitview_client.BitviewClient.get_block_by_timestamp"></a>

#### get\_block\_by\_timestamp

```python
def get_block_by_timestamp(timestamp: Timestamp) -> BlockTimestamp
```

Block by timestamp.

Find the block closest to a given UNIX timestamp.

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

Retrieve the status of a block. Returns whether the block is in the best chain and, if so, its height and the hash of the next block.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-status)*

Endpoint: `GET /api/block/{hash}/status`

<a id="bitview_client.BitviewClient.get_block_tip_height"></a>

#### get\_block\_tip\_height

```python
def get_block_tip_height() -> Height
```

Block tip height.

Returns the height of the last block.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-tip-height)*

Endpoint: `GET /api/blocks/tip/height`

<a id="bitview_client.BitviewClient.get_block_tip_hash"></a>

#### get\_block\_tip\_hash

```python
def get_block_tip_hash() -> BlockHash
```

Block tip hash.

Returns the hash of the last block.

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

Retrieve all transaction IDs in a block. Returns an array of txids in block order.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transaction-ids)*

Endpoint: `GET /api/block/{hash}/txids`

<a id="bitview_client.BitviewClient.get_block_txs"></a>

#### get\_block\_txs

```python
def get_block_txs(hash: BlockHash) -> List[Transaction]
```

Block transactions.

Retrieve transactions in a block by block hash. Returns up to 25 transactions starting from index 0.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transactions)*

Endpoint: `GET /api/block/{hash}/txs`

<a id="bitview_client.BitviewClient.get_block_txs_from_index"></a>

#### get\_block\_txs\_from\_index

```python
def get_block_txs_from_index(hash: BlockHash,
                             start_index: BlockTxIndex) -> List[Transaction]
```

Block transactions (paginated).

Retrieve transactions in a block by block hash, starting from the specified index. Returns up to 25 transactions at a time.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transactions)*

Endpoint: `GET /api/block/{hash}/txs/{start_index}`

<a id="bitview_client.BitviewClient.get_blocks"></a>

#### get\_blocks

```python
def get_blocks() -> List[BlockInfo]
```

Recent blocks.

Retrieve the last 10 blocks. Returns block metadata for each block.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks)*

Endpoint: `GET /api/blocks`

<a id="bitview_client.BitviewClient.get_blocks_from_height"></a>

#### get\_blocks\_from\_height

```python
def get_blocks_from_height(height: Height) -> List[BlockInfo]
```

Blocks from height.

Retrieve up to 10 blocks going backwards from the given height. For example, height=100 returns blocks 100, 99, 98, ..., 91. Height=0 returns only block 0.

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

Retrieve up to 15 blocks with extended data going backwards from the given height.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks-v1)*

Endpoint: `GET /api/v1/blocks/{height}`

<a id="bitview_client.BitviewClient.get_pools"></a>

#### get\_pools

```python
def get_pools() -> List[PoolInfo]
```

List all mining pools.

Get list of all known mining pools with their identifiers.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pools)*

Endpoint: `GET /api/v1/mining/pools`

<a id="bitview_client.BitviewClient.get_pool_stats"></a>

#### get\_pool\_stats

```python
def get_pool_stats(time_period: TimePeriod) -> PoolsSummary
```

Mining pool statistics.

Get mining pool statistics for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pools)*

Endpoint: `GET /api/v1/mining/pools/{time_period}`

<a id="bitview_client.BitviewClient.get_pool"></a>

#### get\_pool

```python
def get_pool(slug: PoolSlug) -> PoolDetail
```

Mining pool details.

Get detailed information about a specific mining pool including block counts and shares for different time periods.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool)*

Endpoint: `GET /api/v1/mining/pool/{slug}`

<a id="bitview_client.BitviewClient.get_pools_hashrate"></a>

#### get\_pools\_hashrate

```python
def get_pools_hashrate() -> List[PoolHashrateEntry]
```

All pools hashrate (all time).

Get hashrate data for all mining pools.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-hashrates)*

Endpoint: `GET /api/v1/mining/hashrate/pools`

<a id="bitview_client.BitviewClient.get_pools_hashrate_by_period"></a>

#### get\_pools\_hashrate\_by\_period

```python
def get_pools_hashrate_by_period(
        time_period: TimePeriod) -> List[PoolHashrateEntry]
```

All pools hashrate.

Get hashrate data for all mining pools for a time period. Valid periods: `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.

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

Get the 10 most recent blocks mined by a specific pool.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-blocks)*

Endpoint: `GET /api/v1/mining/pool/{slug}/blocks`

<a id="bitview_client.BitviewClient.get_pool_blocks_from"></a>

#### get\_pool\_blocks\_from

```python
def get_pool_blocks_from(slug: PoolSlug, height: Height) -> List[BlockInfoV1]
```

Mining pool blocks from height.

Get 10 blocks mined by a specific pool before (and including) the given height.

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

Get network hashrate and difficulty data for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-hashrate)*

Endpoint: `GET /api/v1/mining/hashrate/{time_period}`

<a id="bitview_client.BitviewClient.get_difficulty_adjustments"></a>

#### get\_difficulty\_adjustments

```python
def get_difficulty_adjustments() -> List[DifficultyAdjustmentEntry]
```

Difficulty adjustments (all time).

Get historical difficulty adjustments including timestamp, block height, difficulty value, and percentage change.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustments)*

Endpoint: `GET /api/v1/mining/difficulty-adjustments`

<a id="bitview_client.BitviewClient.get_difficulty_adjustments_by_period"></a>

#### get\_difficulty\_adjustments\_by\_period

```python
def get_difficulty_adjustments_by_period(
        time_period: TimePeriod) -> List[DifficultyAdjustmentEntry]
```

Difficulty adjustments.

Get historical difficulty adjustments for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.

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

Get average total fees per block for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-fees)*

Endpoint: `GET /api/v1/mining/blocks/fees/{time_period}`

<a id="bitview_client.BitviewClient.get_block_rewards"></a>

#### get\_block\_rewards

```python
def get_block_rewards(time_period: TimePeriod) -> List[BlockRewardsEntry]
```

Block rewards.

Get average coinbase reward (subsidy + fees) per block for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-rewards)*

Endpoint: `GET /api/v1/mining/blocks/rewards/{time_period}`

<a id="bitview_client.BitviewClient.get_block_fee_rates"></a>

#### get\_block\_fee\_rates

```python
def get_block_fee_rates(time_period: TimePeriod) -> List[BlockFeeRatesEntry]
```

Block fee rates.

Get block fee rate percentiles (min, 10th, 25th, median, 75th, 90th, max) for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-feerates)*

Endpoint: `GET /api/v1/mining/blocks/fee-rates/{time_period}`

<a id="bitview_client.BitviewClient.get_block_sizes_weights"></a>

#### get\_block\_sizes\_weights

```python
def get_block_sizes_weights(time_period: TimePeriod) -> BlockSizesWeights
```

Block sizes and weights.

Get average block sizes and weights for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.

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

Recommended fee rates by confirmation target.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-recommended-fees)*

Endpoint: `GET /api/v1/fees/recommended`

<a id="bitview_client.BitviewClient.get_precise_fees"></a>

#### get\_precise\_fees

```python
def get_precise_fees() -> RecommendedFees
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

Get current mempool statistics including transaction count, total vsize, total fees, and fee histogram.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool)*

Endpoint: `GET /api/mempool`

<a id="bitview_client.BitviewClient.get_mempool_hash"></a>

#### get\_mempool\_hash

```python
def get_mempool_hash() -> NextBlockHash
```

Mempool content hash.

Returns an opaque hash that changes whenever the projected next block changes. Same value as the mempool ETag. Useful as a freshness/liveness signal: if it stays constant for tens of seconds on a live network, the mempool sync loop has stalled.

Endpoint: `GET /api/mempool/hash`

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

Returns up to 25 most-recent RBF replacement trees across the whole mempool. Each entry has the same shape as `tx_rbf().replacements`.

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

<a id="bitview_client.BitviewClient.get_block_template"></a>

#### get\_block\_template

```python
def get_block_template() -> BlockTemplate
```

Projected next block template.

Bitcoin Core's `getblocktemplate` selection: full transaction bodies in GBT order with aggregate stats. The returned `hash` is an opaque content token; pass it to `GET /api/v1/mempool/block-template/diff/{hash}` to fetch deltas instead of refetching the whole template.

Endpoint: `GET /api/v1/mempool/block-template`

<a id="bitview_client.BitviewClient.get_block_template_diff"></a>

#### get\_block\_template\_diff

```python
def get_block_template_diff(hash: NextBlockHash) -> BlockTemplateDiff
```

Block template diff since hash.

Delta of the projected next block since `<hash>`. `order` is the full new template in order: each entry is either a number (index into the prior template the client cached at `<hash>`) or a transaction object (new body to insert at this position). Walk `order` once to rebuild; `removed` is a convenience list of txids that left so clients can evict cached bodies. After applying, use the response `hash` as `<hash>` on the next call to keep iterating. Returns `404` when `<hash>` has aged out of server history; clients should fall back to `GET /api/v1/mempool/block-template`.

Endpoint: `GET /api/v1/mempool/block-template/diff/{hash}`

<a id="bitview_client.BitviewClient.get_live_price"></a>

#### get\_live\_price

```python
def get_live_price() -> Dollars
```

Live BTC/USD price.

Returns the current BTC/USD price in dollars, derived from on-chain round-dollar output patterns in the last 12 blocks plus mempool.

Endpoint: `GET /api/mempool/price`

<a id="bitview_client.BitviewClient.get_oracle_price"></a>

#### get\_oracle\_price

```python
def get_oracle_price() -> Dollars
```

Live BTC/USD price.

Current BTC/USD price in dollars. Same value as `GET /api/mempool/price`. Confirmed per-height history is available at `GET /api/series/price/height`.

Endpoint: `GET /api/oracle/price`

<a id="bitview_client.BitviewClient.get_oracle_histogram_payments_live"></a>

#### get\_oracle\_histogram\_payments\_live

```python
def get_oracle_histogram_payments_live() -> List[int]
```

Live payment output histogram.

Live smoothed histogram of oracle-eligible payment outputs, binned by output value on the oracle log scale. It combines the committed oracle window with the forming mempool block. A flat array of log-scale bins.

Endpoint: `GET /api/oracle/histogram/payments/live`

<a id="bitview_client.BitviewClient.get_oracle_histogram_payments"></a>

#### get\_oracle\_histogram\_payments

```python
def get_oracle_histogram_payments(point: str) -> List[int]
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

Live unfiltered output value histogram for the forming mempool block. Every live output is binned by value on the oracle log scale; no oracle payment filters are applied. A flat array of log-scale bins, all zero when no mempool is configured.

Endpoint: `GET /api/oracle/histogram/outputs/live`

<a id="bitview_client.BitviewClient.get_oracle_histogram_outputs"></a>

#### get\_oracle\_histogram\_outputs

```python
def get_oracle_histogram_outputs(point: str) -> List[int]
```

Output value histogram at height or day.

Unfiltered output value histogram for a confirmed point. A block height (`840000`) gives every output in that block, coinbase included, binned by value on the oracle log scale; a calendar date (`YYYY-MM-DD`) sums every block that day. A flat array of log-scale bins.

Endpoint: `GET /api/oracle/histogram/outputs/{point}`

<a id="bitview_client.BitviewClient.get_tx_by_index"></a>

#### get\_tx\_by\_index

```python
def get_tx_by_index(index: TxIndex) -> Txid
```

Txid by index.

Retrieve the transaction ID (txid) at a given global transaction index. Returns the txid as plain text.

Endpoint: `GET /api/tx-index/{index}`

<a id="bitview_client.BitviewClient.get_cpfp"></a>

#### get\_cpfp

```python
def get_cpfp(txid: Txid) -> CpfpInfo
```

CPFP info.

Returns ancestors and descendants for a CPFP (Child Pays For Parent) transaction, including the effective fee rate of the package.

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

Retrieve complete transaction data by transaction ID (txid). Returns inputs, outputs, fee, size, and confirmation status.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction)*

Endpoint: `GET /api/tx/{txid}`

<a id="bitview_client.BitviewClient.get_tx_hex"></a>

#### get\_tx\_hex

```python
def get_tx_hex(txid: Txid) -> Hex
```

Transaction hex.

Retrieve the raw transaction as a hex-encoded string. Returns the serialized transaction in hexadecimal format.

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

Get the merkle inclusion proof for a transaction.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-merkle-proof)*

Endpoint: `GET /api/tx/{txid}/merkle-proof`

<a id="bitview_client.BitviewClient.get_tx_outspend"></a>

#### get\_tx\_outspend

```python
def get_tx_outspend(txid: Txid, vout: Vout) -> TxOutspend
```

Output spend status.

Get the spending status of a transaction output. Returns whether the output has been spent and, if so, the spending transaction details.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-outspend)*

Endpoint: `GET /api/tx/{txid}/outspend/{vout}`

<a id="bitview_client.BitviewClient.get_tx_outspends"></a>

#### get\_tx\_outspends

```python
def get_tx_outspends(txid: Txid) -> List[TxOutspend]
```

All output spend statuses.

Get the spending status of all outputs in a transaction. Returns an array with the spend status for each output.

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

Retrieve the confirmation status of a transaction. Returns whether the transaction is confirmed and, if so, the block height, hash, and timestamp.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-status)*

Endpoint: `GET /api/tx/{txid}/status`

<a id="bitview_client.BitviewClient.get_transaction_times"></a>

#### get\_transaction\_times

```python
def get_transaction_times(txId: List[Txid]) -> List[int]
```

Transaction first-seen times.

Returns timestamps when transactions were first seen in the mempool. Returns 0 for mined or unknown transactions.

*[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-times)*

Endpoint: `GET /api/v1/transaction-times`

<a id="bitview_client.BitviewClient.post_tx"></a>

#### post\_tx

```python
def post_tx(body: str) -> Txid
```

Broadcast transaction.

Broadcast a raw transaction to the network. The transaction should be provided as hex in the request body. The txid will be returned on success.

*[Mempool.space docs](https://mempool.space/docs/api/rest#post-transaction)*

Endpoint: `POST /api/tx`

<a id="bitview_client.BitviewClient.get_openapi"></a>

#### get\_openapi

```python
def get_openapi() -> str
```

OpenAPI specification.

Full OpenAPI 3.1 specification for this API.

Endpoint: `GET /openapi.json`

<a id="bitview_client.BitviewClient.get_api"></a>

#### get\_api

```python
def get_api() -> Any
```

Compact OpenAPI specification.

Compact OpenAPI specification optimized for LLM consumption. Removes redundant fields while preserving essential API information. The full specification is available at `GET /openapi.json`.

Endpoint: `GET /api.json`

