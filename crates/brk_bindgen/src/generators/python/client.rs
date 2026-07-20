//! Python base client and pattern factory generation.

use std::fmt::Write;

use crate::{
    ClientConstants, ClientMetadata, CohortConstants, IndexSetPattern, PythonSyntax,
    StructuralPattern, format_json, generate_parameterized_field, index_to_field_name,
};

/// Generate class-level constants for the BrkClient class.
pub fn generate_class_constants(output: &mut String) {
    let constants = ClientConstants::collect();

    // VERSION
    writeln!(output, "    VERSION = \"{}\"\n", constants.version).unwrap();

    // INDEXES, POOL_ID_TO_POOL_NAME
    write_class_const(output, "INDEXES", &format_json(&constants.indexes));
    // Python needs string keys for pool map
    let pool_map: std::collections::BTreeMap<String, &str> = constants
        .pool_map
        .iter()
        .map(|(k, v)| (k.to_string(), *v))
        .collect();
    write_class_const(output, "POOL_ID_TO_POOL_NAME", &format_json(&pool_map));

    // Cohort constants (no camelCase conversion for Python)
    for (name, value) in CohortConstants::all() {
        write_class_const(output, name, &format_json(&value));
    }
}

fn write_class_const(output: &mut String, name: &str, json: &str) {
    let indented = json
        .lines()
        .enumerate()
        .map(|(i, line)| {
            if i == 0 {
                format!("    {} = {}", name, line)
            } else {
                format!("    {}", line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    writeln!(output, "{}\n", indented).unwrap();
}

/// Generate the base BrkClient class with HTTP functionality
pub fn generate_base_client(output: &mut String) {
    writeln!(
        output,
        r#"class BrkError(Exception):
    """Custom error class for BRK client errors."""

    def __init__(self, message: str, status: Optional[int] = None):
        super().__init__(message)
        self.status = status


class BrkClientBase:
    """Base HTTP client for making requests."""

    def __init__(self, base_url: str, timeout: float = 30.0):
        parsed = urlparse(base_url)
        self._host = parsed.netloc
        self._secure = parsed.scheme == 'https'
        self._timeout = timeout
        self._conn: Optional[Union[HTTPSConnection, HTTPConnection]] = None

    def _connect(self) -> Union[HTTPSConnection, HTTPConnection]:
        """Get or create HTTP connection."""
        if self._conn is None:
            if self._secure:
                self._conn = HTTPSConnection(self._host, timeout=self._timeout)
            else:
                self._conn = HTTPConnection(self._host, timeout=self._timeout)
        return self._conn

    def get(self, path: str) -> bytes:
        """Make a GET request and return raw bytes."""
        try:
            conn = self._connect()
            conn.request("GET", path)
            res = conn.getresponse()
            data = res.read()
            if res.status >= 400:
                raise BrkError(f"HTTP error: {{res.status}}", res.status)
            return data
        except (ConnectionError, OSError, TimeoutError) as e:
            self._conn = None
            raise BrkError(str(e))

    def get_json(self, path: str) -> Any:
        """Make a GET request and return JSON."""
        return json.loads(self.get(path))

    def get_text(self, path: str) -> str:
        """Make a GET request and return text."""
        return self.get(path).decode()

    def post(self, path: str, body: str) -> bytes:
        """Make a POST request with a string body and return raw bytes."""
        try:
            conn = self._connect()
            conn.request("POST", path, body=body)
            res = conn.getresponse()
            data = res.read()
            if res.status >= 400:
                raise BrkError(f"HTTP error: {{res.status}}", res.status)
            return data
        except (ConnectionError, OSError, TimeoutError) as e:
            self._conn = None
            raise BrkError(str(e))

    def post_json(self, path: str, body: str) -> Any:
        """Make a POST request and return JSON."""
        return json.loads(self.post(path, body))

    def post_text(self, path: str, body: str) -> str:
        """Make a POST request and return text."""
        return self.post(path, body).decode()

    def close(self) -> None:
        """Close the HTTP client."""
        if self._conn:
            self._conn.close()
            self._conn = None

    def __enter__(self) -> BrkClientBase:
        return self

    def __exit__(self, exc_type: Optional[type], exc_val: Optional[BaseException], exc_tb: Optional[Any]) -> None:
        self.close()


def _m(acc: str, s: str) -> str:
    """Build series name with suffix."""
    if not s: return acc
    return f"{{acc}}_{{s}}" if acc else s


def _p(prefix: str, acc: str) -> str:
    """Build series name with prefix."""
    return f"{{prefix}}_{{acc}}" if acc else prefix

"#
    )
    .unwrap();
    output.push_str(r#"
_MASK_64 = (1 << 64) - 1
_RAPIDHASH_SECRETS = (
    0x2d358dccaa6c78a5,
    0x8bb84b93962eacc9,
    0x4b33a62ed433d4a3,
    0x4d5a2da51de1aa47,
    0xa0761d6478bd642f,
    0xe7037ed1a0b428db,
    0x90ed1765281c388c,
)
_RAPIDHASH_SEED = 0


def _u64(value: int) -> int:
    return value & _MASK_64


def _rapid_mix(left: int, right: int) -> int:
    result = _u64(left) * _u64(right)
    return _u64(result) ^ _u64(result >> 64)


def _rapid_mum(left: int, right: int) -> Tuple[int, int]:
    result = _u64(left) * _u64(right)
    return _u64(result), _u64(result >> 64)


def _rapid_hash_seed(seed: int) -> int:
    return _u64(seed ^ _rapid_mix(seed ^ _RAPIDHASH_SECRETS[2], _RAPIDHASH_SECRETS[1]))


_RAPIDHASH_SEED = _rapid_hash_seed(0)


def _read_u32(data: bytes, offset: int) -> int:
    return int.from_bytes(data[offset:offset + 4], "little")


def _read_u64(data: bytes, offset: int) -> int:
    return int.from_bytes(data[offset:offset + 8], "little")


def _rapid_hash_v3(payload: Union[bytes, bytearray, memoryview]) -> int:
    data = bytes(payload)
    length = len(data)
    if length == 0:
        raise ValueError("Expected a non-empty address payload")
    if length > 65:
        raise ValueError("Expected at most 65 address payload bytes")

    seed = _RAPIDHASH_SEED
    a = 0
    b = 0

    if length <= 16:
        if length >= 4:
            seed ^= length
            if length >= 8:
                a ^= _read_u64(data, 0)
                b ^= _read_u64(data, length - 8)
            else:
                a ^= _read_u32(data, 0)
                b ^= _read_u32(data, length - 4)
        elif length > 0:
            a ^= (data[0] << 45) | data[length - 1]
            b ^= data[length >> 1]
        remainder = length
    else:
        if length > 16:
            seed = _rapid_mix(_read_u64(data, 0) ^ _RAPIDHASH_SECRETS[2], _read_u64(data, 8) ^ seed)
            if length > 32:
                seed = _rapid_mix(_read_u64(data, 16) ^ _RAPIDHASH_SECRETS[2], _read_u64(data, 24) ^ seed)
                if length > 48:
                    seed = _rapid_mix(_read_u64(data, 32) ^ _RAPIDHASH_SECRETS[1], _read_u64(data, 40) ^ seed)
                    if length > 64:
                        seed = _rapid_mix(_read_u64(data, 48) ^ _RAPIDHASH_SECRETS[1], _read_u64(data, 56) ^ seed)
        remainder = length
        a ^= _read_u64(data, length - 16) ^ remainder
        b ^= _read_u64(data, length - 8)

    a ^= _RAPIDHASH_SECRETS[1]
    b ^= seed
    a, b = _rapid_mum(a, b)
    return _rapid_mix(a ^ 0xaaaaaaaaaaaaaaaa, b ^ _RAPIDHASH_SECRETS[1] ^ remainder)


def _validate_hash_prefix_nibbles(nibbles: int) -> None:
    if isinstance(nibbles, bool) or not isinstance(nibbles, int) or nibbles < 1 or nibbles > 16:
        raise ValueError("Expected hash-prefix length from 1 to 16 hex nibbles")


def _address_payload_lengths(addr_type: OutputType) -> Tuple[int, ...]:
    if addr_type == "p2a":
        return (2,)
    if addr_type == "p2pk33":
        return (33,)
    if addr_type == "p2pk65":
        return (65,)
    if addr_type in ("p2pkh", "p2sh", "p2wpkh"):
        return (20,)
    if addr_type in ("p2wsh", "p2tr"):
        return (32,)
    raise ValueError(f"Unsupported address type for address payload hash-prefix: {addr_type}")


def _validate_address_payload_for_type(addr_type: OutputType, payload: Union[bytes, bytearray, memoryview]) -> None:
    length = len(bytes(payload))
    expected = _address_payload_lengths(addr_type)
    if length not in expected:
        joined = " or ".join(str(value) for value in expected)
        raise ValueError(f"Expected {addr_type} address payload length {joined} bytes")


def address_payload_hash_prefix(payload: Union[bytes, bytearray, memoryview], nibbles: int) -> str:
    """Compute the RapidHash v3 hash-prefix used by `/api/address/hash-prefix/{addr_type}/{prefix}`."""
    _validate_hash_prefix_nibbles(nibbles)
    return f"{_rapid_hash_v3(payload):016x}"[:nibbles]


"#);
}

/// Generate the SeriesData and SeriesEndpoint classes
pub fn generate_endpoint_class(output: &mut String) {
    writeln!(
        output,
        r#"# Date conversion constants
_GENESIS = date(2009, 1, 3)  # day1 0, week1 0
_DAY_ONE = date(2009, 1, 9)  # day1 1 (6 day gap after genesis)
_EPOCH = datetime(2009, 1, 1, tzinfo=timezone.utc)
_DATE_INDEXES = frozenset([
    'minute10', 'minute30',
    'hour1', 'hour4', 'hour12',
    'day1', 'day3', 'week1',
    'month1', 'month3', 'month6',
    'year1', 'year10',
])

def _index_to_date(index: str, i: int) -> Union[date, datetime]:
    """Convert an index value to a date/datetime for date-based indexes."""
    if index == 'minute10':
        return _EPOCH + timedelta(minutes=i * 10)
    elif index == 'minute30':
        return _EPOCH + timedelta(minutes=i * 30)
    elif index == 'hour1':
        return _EPOCH + timedelta(hours=i)
    elif index == 'hour4':
        return _EPOCH + timedelta(hours=i * 4)
    elif index == 'hour12':
        return _EPOCH + timedelta(hours=i * 12)
    elif index == 'day1':
        return _GENESIS if i == 0 else _DAY_ONE + timedelta(days=i - 1)
    elif index == 'day3':
        return _EPOCH.date() - timedelta(days=1) + timedelta(days=i * 3)
    elif index == 'week1':
        return _GENESIS + timedelta(weeks=i)
    elif index == 'month1':
        return date(2009 + i // 12, i % 12 + 1, 1)
    elif index == 'month3':
        m = i * 3
        return date(2009 + m // 12, m % 12 + 1, 1)
    elif index == 'month6':
        m = i * 6
        return date(2009 + m // 12, m % 12 + 1, 1)
    elif index == 'year1':
        return date(2009 + i, 1, 1)
    elif index == 'year10':
        return date(2009 + i * 10, 1, 1)
    else:
        raise ValueError(f"{{index}} is not a date-based index")


def _date_to_index(index: str, d: Union[date, datetime]) -> int:
    """Convert a date/datetime to an index value for date-based indexes.

    Returns the floor index (latest index whose date is <= the given date).
    For sub-day indexes (minute*, hour*), a plain date is treated as midnight UTC.
    """
    if index in ('minute10', 'minute30', 'hour1', 'hour4', 'hour12'):
        if isinstance(d, datetime):
            dt = d if d.tzinfo else d.replace(tzinfo=timezone.utc)
        else:
            dt = datetime(d.year, d.month, d.day, tzinfo=timezone.utc)
        secs = int((dt - _EPOCH).total_seconds())
        div = {{'minute10': 600, 'minute30': 1800,
               'hour1': 3600, 'hour4': 14400, 'hour12': 43200}}
        return secs // div[index]
    dd = d.date() if isinstance(d, datetime) else d
    if index == 'day1':
        if dd < _DAY_ONE:
            return 0
        return 1 + (dd - _DAY_ONE).days
    elif index == 'day3':
        return (dd - date(2008, 12, 31)).days // 3
    elif index == 'week1':
        return (dd - _GENESIS).days // 7
    elif index == 'month1':
        return (dd.year - 2009) * 12 + (dd.month - 1)
    elif index == 'month3':
        return (dd.year - 2009) * 4 + (dd.month - 1) // 3
    elif index == 'month6':
        return (dd.year - 2009) * 2 + (dd.month - 1) // 6
    elif index == 'year1':
        return dd.year - 2009
    elif index == 'year10':
        return (dd.year - 2009) // 10
    else:
        raise ValueError(f"{{index}} is not a date-based index")


@dataclass
class SeriesData(Generic[T]):
    """Series data with range information. Always int-indexed."""
    version: int
    index: Index
    type: str
    start: int
    end: int
    stamp: str
    data: List[T]

    @property
    def is_date_based(self) -> bool:
        """Whether this series uses a date-based index."""
        return self.index in _DATE_INDEXES

    def indexes(self) -> List[int]:
        """Get raw index numbers."""
        return list(range(self.start, self.end))

    def keys(self) -> List[int]:
        """Get keys as index numbers."""
        return self.indexes()

    def items(self) -> List[Tuple[int, T]]:
        """Get (index, value) pairs."""
        return list(zip(self.indexes(), self.data))

    def to_dict(self) -> Dict[int, T]:
        """Return {{index: value}} dict."""
        return dict(zip(self.indexes(), self.data))

    def __iter__(self) -> Iterator[Tuple[int, T]]:
        """Iterate over (index, value) pairs."""
        return iter(zip(self.indexes(), self.data))

    def __len__(self) -> int:
        return len(self.data)

    def to_polars(self) -> pl.DataFrame:
        """Convert to Polars DataFrame with 'index' and 'value' columns."""
        try:
            import polars as pl  # type: ignore[import-not-found]
        except ImportError:
            raise ImportError("polars is required: pip install polars")
        return pl.DataFrame({{"index": self.indexes(), "value": self.data}})

    def to_pandas(self) -> pd.DataFrame:
        """Convert to Pandas DataFrame with 'index' and 'value' columns."""
        try:
            import pandas as pd  # type: ignore[import-not-found]
        except ImportError:
            raise ImportError("pandas is required: pip install pandas")
        return pd.DataFrame({{"index": self.indexes(), "value": self.data}})


@dataclass
class DateSeriesData(SeriesData[T]):
    """Series data with date-based index. Extends SeriesData with date methods."""

    def dates(self) -> List[Union[date, datetime]]:
        """Get dates for the index range. Returns datetime for sub-daily indexes, date for daily+."""
        return [_index_to_date(self.index, i) for i in range(self.start, self.end)]

    def date_items(self) -> List[Tuple[Union[date, datetime], T]]:
        """Get (date, value) pairs."""
        return list(zip(self.dates(), self.data))

    def to_date_dict(self) -> Dict[Union[date, datetime], T]:
        """Return {{date: value}} dict."""
        return dict(zip(self.dates(), self.data))

    def to_polars(self, with_dates: bool = True) -> pl.DataFrame:
        """Convert to Polars DataFrame.

        Returns a DataFrame with columns:
        - 'date' and 'value' if with_dates=True (default)
        - 'index' and 'value' otherwise
        """
        try:
            import polars as pl  # type: ignore[import-not-found]
        except ImportError:
            raise ImportError("polars is required: pip install polars")
        if with_dates:
            return pl.DataFrame({{"date": self.dates(), "value": self.data}})
        return pl.DataFrame({{"index": self.indexes(), "value": self.data}})

    def to_pandas(self, with_dates: bool = True) -> pd.DataFrame:
        """Convert to Pandas DataFrame.

        Returns a DataFrame with columns:
        - 'date' and 'value' if with_dates=True (default)
        - 'index' and 'value' otherwise
        """
        try:
            import pandas as pd  # type: ignore[import-not-found]
        except ImportError:
            raise ImportError("pandas is required: pip install pandas")
        if with_dates:
            return pd.DataFrame({{"date": self.dates(), "value": self.data}})
        return pd.DataFrame({{"index": self.indexes(), "value": self.data}})


# Type aliases for non-generic usage
AnySeriesData = SeriesData[Any]
AnyDateSeriesData = DateSeriesData[Any]


class _EndpointConfig:
    """Shared endpoint configuration."""
    client: BrkClient
    name: str
    index: Index
    start: Optional[int]
    end: Optional[int]

    def __init__(self, client: BrkClient, name: str, index: Index,
                 start: Optional[int] = None, end: Optional[int] = None):
        self.client = client
        self.name = name
        self.index = index
        self.start = start
        self.end = end

    def path(self) -> str:
        return f"/api/series/{{self.name}}/{{self.index}}"

    def _build_path(self, format: Optional[str] = None) -> str:
        params = []
        if self.start is not None:
            params.append(f"start={{self.start}}")
        if self.end is not None:
            params.append(f"end={{self.end}}")
        if format is not None:
            params.append(f"format={{format}}")
        query = "&".join(params)
        p = self.path()
        return f"{{p}}?{{query}}" if query else p

    def _new(self, start: Optional[int] = None, end: Optional[int] = None) -> _EndpointConfig:
        return _EndpointConfig(self.client, self.name, self.index, start, end)

    def get_series(self) -> SeriesData[Any]:
        return SeriesData(**self.client.get_json(self._build_path()))

    def get_date_series(self) -> DateSeriesData[Any]:
        return DateSeriesData(**self.client.get_json(self._build_path()))

    def get_csv(self) -> str:
        return self.client.get_text(self._build_path(format='csv'))

    def get_len(self) -> int:
        return self.client.get_series_len(self.name, self.index)

    def get_version(self) -> Version:
        return self.client.get_series_version(self.name, self.index)


class RangeBuilder(Generic[T]):
    """Builder with range specified."""

    def __init__(self, config: _EndpointConfig):
        self._config = config

    def fetch(self) -> SeriesData[T]:
        """Fetch the range as parsed JSON."""
        return self._config.get_series()

    def fetch_csv(self) -> str:
        """Fetch the range as CSV string."""
        return self._config.get_csv()


class SingleItemBuilder(Generic[T]):
    """Builder for single item access."""

    def __init__(self, config: _EndpointConfig):
        self._config = config

    def fetch(self) -> SeriesData[T]:
        """Fetch the single item."""
        return self._config.get_series()

    def fetch_csv(self) -> str:
        """Fetch as CSV."""
        return self._config.get_csv()


class SkippedBuilder(Generic[T]):
    """Builder after calling skip(n). Chain with take() to specify count."""

    def __init__(self, config: _EndpointConfig):
        self._config = config

    def take(self, n: int) -> RangeBuilder[T]:
        """Take n items after the skipped position."""
        start = self._config.start or 0
        return RangeBuilder(self._config._new(start, start + n))

    def fetch(self) -> SeriesData[T]:
        """Fetch from skipped position to end."""
        return self._config.get_series()

    def fetch_csv(self) -> str:
        """Fetch as CSV."""
        return self._config.get_csv()


class DateRangeBuilder(RangeBuilder[T]):
    """Range builder that returns DateSeriesData."""
    def fetch(self) -> DateSeriesData[T]:
        return self._config.get_date_series()


class DateSingleItemBuilder(SingleItemBuilder[T]):
    """Single item builder that returns DateSeriesData."""
    def fetch(self) -> DateSeriesData[T]:
        return self._config.get_date_series()


class DateSkippedBuilder(SkippedBuilder[T]):
    """Skipped builder that returns DateSeriesData."""
    def take(self, n: int) -> DateRangeBuilder[T]:
        start = self._config.start or 0
        return DateRangeBuilder(self._config._new(start, start + n))
    def fetch(self) -> DateSeriesData[T]:
        return self._config.get_date_series()


class SeriesEndpoint(Generic[T]):
    """Builder for series endpoint queries with int-based indexing.

    Examples:
        data = endpoint.fetch()
        data = endpoint[5].fetch()
        data = endpoint[:10].fetch()
        data = endpoint.head(20).fetch()
        data = endpoint.skip(100).take(10).fetch()
    """

    def __init__(self, client: BrkClient, name: str, index: Index):
        self._config = _EndpointConfig(client, name, index)

    @overload
    def __getitem__(self, key: int) -> SingleItemBuilder[T]: ...
    @overload
    def __getitem__(self, key: slice) -> RangeBuilder[T]: ...

    def __getitem__(self, key: Union[int, slice]) -> Union[SingleItemBuilder[T], RangeBuilder[T]]:
        """Access single item or slice by integer index."""
        if isinstance(key, int):
            return SingleItemBuilder(self._config._new(key, key + 1))
        return RangeBuilder(self._config._new(key.start, key.stop))

    def head(self, n: int = 10) -> RangeBuilder[T]:
        """Get the first n items."""
        return RangeBuilder(self._config._new(end=n))

    def tail(self, n: int = 10) -> RangeBuilder[T]:
        """Get the last n items."""
        return RangeBuilder(self._config._new(end=0) if n == 0 else self._config._new(start=-n))

    def skip(self, n: int) -> SkippedBuilder[T]:
        """Skip the first n items."""
        return SkippedBuilder(self._config._new(start=n))

    def fetch(self) -> SeriesData[T]:
        """Fetch all data."""
        return self._config.get_series()

    def fetch_csv(self) -> str:
        """Fetch all data as CSV."""
        return self._config.get_csv()

    def len(self) -> int:
        """Total number of data points for this series."""
        return self._config.get_len()

    def version(self) -> Version:
        """Current version of the series."""
        return self._config.get_version()

    def path(self) -> str:
        """Get the base endpoint path."""
        return self._config.path()


class DateSeriesEndpoint(Generic[T]):
    """Builder for series endpoint queries with date-based indexing.

    Accepts dates in __getitem__ and returns DateSeriesData from fetch().

    Examples:
        data = endpoint.fetch()
        data = endpoint[date(2020, 1, 1)].fetch()
        data = endpoint[date(2020, 1, 1):date(2023, 1, 1)].fetch()
        data = endpoint[:10].fetch()
    """

    def __init__(self, client: BrkClient, name: str, index: Index):
        self._config = _EndpointConfig(client, name, index)

    @overload
    def __getitem__(self, key: int) -> DateSingleItemBuilder[T]: ...
    @overload
    def __getitem__(self, key: datetime) -> DateSingleItemBuilder[T]: ...
    @overload
    def __getitem__(self, key: date) -> DateSingleItemBuilder[T]: ...
    @overload
    def __getitem__(self, key: slice) -> DateRangeBuilder[T]: ...

    def __getitem__(self, key: Union[int, slice, date, datetime]) -> Union[DateSingleItemBuilder[T], DateRangeBuilder[T]]:
        """Access single item or slice. Accepts int, date, or datetime."""
        if isinstance(key, (date, datetime)):
            idx = _date_to_index(self._config.index, key)
            return DateSingleItemBuilder(self._config._new(idx, idx + 1))
        if isinstance(key, int):
            return DateSingleItemBuilder(self._config._new(key, key + 1))
        start, stop = key.start, key.stop
        if isinstance(start, (date, datetime)):
            start = _date_to_index(self._config.index, start)
        if isinstance(stop, (date, datetime)):
            stop = _date_to_index(self._config.index, stop)
        return DateRangeBuilder(self._config._new(start, stop))

    def head(self, n: int = 10) -> DateRangeBuilder[T]:
        """Get the first n items."""
        return DateRangeBuilder(self._config._new(end=n))

    def tail(self, n: int = 10) -> DateRangeBuilder[T]:
        """Get the last n items."""
        return DateRangeBuilder(self._config._new(end=0) if n == 0 else self._config._new(start=-n))

    def skip(self, n: int) -> DateSkippedBuilder[T]:
        """Skip the first n items."""
        return DateSkippedBuilder(self._config._new(start=n))

    def fetch(self) -> DateSeriesData[T]:
        """Fetch all data."""
        return self._config.get_date_series()

    def fetch_csv(self) -> str:
        """Fetch all data as CSV."""
        return self._config.get_csv()

    def len(self) -> int:
        """Total number of data points for this series."""
        return self._config.get_len()

    def version(self) -> Version:
        """Current version of the series."""
        return self._config.get_version()

    def path(self) -> str:
        """Get the base endpoint path."""
        return self._config.path()


# Type aliases for non-generic usage
AnySeriesEndpoint = SeriesEndpoint[Any]
AnyDateSeriesEndpoint = DateSeriesEndpoint[Any]


class SeriesPattern(Protocol[T]):
    """Protocol for series patterns with different index sets."""

    @property
    def name(self) -> str:
        """Get the series name."""
        ...

    def indexes(self) -> List[str]:
        """Get the list of available indexes for this series."""
        ...

    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]:
        """Get an endpoint builder for a specific index, if supported."""
        ...

"#
    )
    .unwrap();
}

/// Generate index accessor classes
pub fn generate_index_accessors(output: &mut String, patterns: &[IndexSetPattern]) {
    if patterns.is_empty() {
        return;
    }

    // Generate static index tuples
    writeln!(output, "# Static index tuples").unwrap();
    for (i, pattern) in patterns.iter().enumerate() {
        write!(output, "_i{} = (", i + 1).unwrap();
        for (j, index) in pattern.indexes.iter().enumerate() {
            if j > 0 {
                write!(output, ", ").unwrap();
            }
            write!(output, "'{}'", index.name()).unwrap();
        }
        // Single-element tuple needs trailing comma
        if pattern.indexes.len() == 1 {
            write!(output, ",").unwrap();
        }
        writeln!(output, ")").unwrap();
    }
    writeln!(output).unwrap();

    // Generate helper functions
    writeln!(
        output,
        r#"def _ep(c: BrkClient, n: str, i: Index) -> SeriesEndpoint[Any]:
    return SeriesEndpoint(c, n, i)

def _dep(c: BrkClient, n: str, i: Index) -> DateSeriesEndpoint[Any]:
    return DateSeriesEndpoint(c, n, i)
"#
    )
    .unwrap();

    writeln!(output, "# Index accessor classes\n").unwrap();

    for (i, pattern) in patterns.iter().enumerate() {
        let by_class_name = format!("_{}By", pattern.name);
        let idx_var = format!("_i{}", i + 1);

        // Generate the By class with compact methods
        writeln!(output, "class {}(Generic[T]):", by_class_name).unwrap();
        writeln!(
            output,
            "    def __init__(self, c: BrkClient, n: str): self._c, self._n = c, n"
        )
        .unwrap();
        for index in &pattern.indexes {
            let method_name = index_to_field_name(index);
            let index_name = index.name();
            let (builder_type, helper) = if index.is_date_based() {
                ("DateSeriesEndpoint", "_dep")
            } else {
                ("SeriesEndpoint", "_ep")
            };
            writeln!(
                output,
                "    def {}(self) -> {}[T]: return {}(self._c, self._n, '{}')",
                method_name, builder_type, helper, index_name
            )
            .unwrap();
        }
        writeln!(output).unwrap();

        // Generate the main accessor class
        writeln!(output, "class {}(Generic[T]):", pattern.name).unwrap();
        writeln!(output, "    by: {}[T]", by_class_name).unwrap();
        writeln!(
            output,
            "    def __init__(self, c: BrkClient, n: str): self._n, self.by = n, {}(c, n)",
            by_class_name
        )
        .unwrap();
        writeln!(output, "    @property").unwrap();
        writeln!(output, "    def name(self) -> str: return self._n").unwrap();
        writeln!(
            output,
            "    def indexes(self) -> List[str]: return list({})",
            idx_var
        )
        .unwrap();
        writeln!(
            output,
            "    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]: return _ep(self.by._c, self._n, index) if index in {} else None",
            idx_var
        )
        .unwrap();
        writeln!(output).unwrap();
    }
}

/// Generate structural pattern classes
pub fn generate_structural_patterns(
    output: &mut String,
    patterns: &[StructuralPattern],
    metadata: &ClientMetadata,
) {
    if patterns.is_empty() {
        return;
    }

    writeln!(output, "# Reusable structural pattern classes\n").unwrap();

    for pattern in patterns {
        // Generate class
        if pattern.is_generic {
            writeln!(output, "class {}(Generic[T]):", pattern.name).unwrap();
        } else {
            writeln!(output, "class {}:", pattern.name).unwrap();
        }
        writeln!(
            output,
            "    \"\"\"Pattern struct for repeated tree structure.\"\"\""
        )
        .unwrap();

        // Skip constructor for non-parameterizable patterns (inlined at tree level)
        if !metadata.is_parameterizable(&pattern.name) {
            writeln!(output, "    pass\n").unwrap();
            continue;
        }

        writeln!(output, "    ").unwrap();
        if pattern.is_templated() {
            writeln!(
                output,
                "    def __init__(self, client: BrkClient, acc: str, disc: str):"
            )
            .unwrap();
        } else {
            writeln!(
                output,
                "    def __init__(self, client: BrkClient, acc: str):"
            )
            .unwrap();
        }
        writeln!(
            output,
            "        \"\"\"Create pattern node with accumulated series name.\"\"\""
        )
        .unwrap();

        let syntax = PythonSyntax;
        for field in &pattern.fields {
            generate_parameterized_field(output, &syntax, field, pattern, metadata, "        ");
        }

        writeln!(output).unwrap();
    }
}
