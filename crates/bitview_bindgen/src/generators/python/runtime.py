class BitviewError(Exception):
    """A failed request: the HTTP status and the server's error code and message, when it sent them."""

    def __init__(self, message: str, status: Optional[int] = None, code: Optional[ErrorCode] = None):
        super().__init__(message)
        self.status = status
        self.code = code


def _response_error(status: int, data: bytes) -> BitviewError:
    """The server's error body as a BitviewError; a body this client can't read keeps its text."""
    text = data.decode(errors="replace")
    try:
        detail = json.loads(text).get("error")
    except (ValueError, AttributeError):
        detail = None
    if not isinstance(detail, dict):
        detail = {}
    message = detail.get("message")
    return BitviewError(
        message if isinstance(message, str) else (text or f"HTTP {status}"),
        status,
        detail.get("code"),
    )


def _segment(value: Any) -> str:
    """A path segment, percent-encoded so `/`, `?` or `#` in a value stay inside it."""
    return quote(str(value), safe='')


class BitviewClientBase:
    def __init__(self, base_url: str, timeout: float = 30.0):
        parsed = urlparse(base_url)
        self._host = parsed.netloc
        self._prefix = parsed.path.rstrip('/')
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

    def _request(self, method: str, path: str, body: Optional[str] = None) -> bytes:
        """A failed exchange closes the connection, which `http.client` would otherwise leave
        mid-request for every later call."""
        try:
            conn = self._connect()
            conn.request(method, self._prefix + path, body=body)
            res = conn.getresponse()
            data = res.read()
        except (HTTPException, OSError) as e:
            self.close()
            raise BitviewError(str(e))
        if res.status >= 400:
            raise _response_error(res.status, data)
        return data

    def get(self, path: str) -> bytes:
        return self._request("GET", path)

    def get_json(self, path: str) -> Any:
        return json.loads(self.get(path))

    def get_text(self, path: str) -> str:
        return self.get(path).decode()

    def post(self, path: str, body: str) -> bytes:
        return self._request("POST", path, body)

    def post_json(self, path: str, body: str) -> Any:
        return json.loads(self.post(path, body))

    def post_text(self, path: str, body: str) -> str:
        return self.post(path, body).decode()

    def close(self) -> None:
        if self._conn:
            self._conn.close()
            self._conn = None

    def __enter__(self) -> BitviewClientBase:
        return self

    def __exit__(self, exc_type: Optional[type], exc_val: Optional[BaseException], exc_tb: Optional[Any]) -> None:
        self.close()


_MASK_64 = (1 << 64) - 1
# RapidHash v3's secrets 1 and 2: the only ones payloads of at most 65 bytes reach.
_RAPIDHASH_SECRET_1 = 0x8bb84b93962eacc9
_RAPIDHASH_SECRET_2 = 0x4b33a62ed433d4a3


def _u64(value: int) -> int:
    return value & _MASK_64


def _rapid_mix(left: int, right: int) -> int:
    result = _u64(left) * _u64(right)
    return _u64(result) ^ _u64(result >> 64)


def _rapid_mum(left: int, right: int) -> Tuple[int, int]:
    result = _u64(left) * _u64(right)
    return _u64(result), _u64(result >> 64)


def _rapid_hash_seed(seed: int) -> int:
    return _u64(seed ^ _rapid_mix(seed ^ _RAPIDHASH_SECRET_2, _RAPIDHASH_SECRET_1))


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
        else:
            a ^= (data[0] << 45) | data[length - 1]
            b ^= data[length >> 1]
        remainder = length
    else:
        seed = _rapid_mix(_read_u64(data, 0) ^ _RAPIDHASH_SECRET_2, _read_u64(data, 8) ^ seed)
        if length > 32:
            seed = _rapid_mix(_read_u64(data, 16) ^ _RAPIDHASH_SECRET_2, _read_u64(data, 24) ^ seed)
            if length > 48:
                seed = _rapid_mix(_read_u64(data, 32) ^ _RAPIDHASH_SECRET_1, _read_u64(data, 40) ^ seed)
                if length > 64:
                    seed = _rapid_mix(_read_u64(data, 48) ^ _RAPIDHASH_SECRET_1, _read_u64(data, 56) ^ seed)
        remainder = length
        a ^= _read_u64(data, length - 16) ^ remainder
        b ^= _read_u64(data, length - 8)

    a ^= _RAPIDHASH_SECRET_1
    b ^= seed
    a, b = _rapid_mum(a, b)
    return _rapid_mix(a ^ 0xaaaaaaaaaaaaaaaa, b ^ _RAPIDHASH_SECRET_1 ^ remainder)


def _validate_hash_prefix_nibbles(nibbles: int) -> None:
    if isinstance(nibbles, bool) or not isinstance(nibbles, int) or nibbles < 1 or nibbles > 16:
        raise ValueError("Expected hash-prefix length from 1 to 16 hex nibbles")


def _validate_address_payload_for_type(address_type: AddressOutputType, payload: Union[bytes, bytearray, memoryview]) -> None:
    length = len(bytes(payload))
    expected = _ADDRESS_PAYLOAD_LENGTHS.get(address_type)
    if expected is None:
        raise ValueError(f"Unsupported address type for address payload hash-prefix: {address_type}")
    if length != expected:
        raise ValueError(f"Expected {address_type} address payload length {expected} bytes")


def address_payload_hash_prefix(payload: Union[bytes, bytearray, memoryview], nibbles: int) -> str:
    """Compute the RapidHash v3 hash-prefix used by `/api/address/hash-prefix/{address_type}/{prefix}`."""
    _validate_hash_prefix_nibbles(nibbles)
    return f"{_rapid_hash_v3(payload):016x}"[:nibbles]


# Date conversion, mirroring the server's indexes (`_CALENDARS`): UTC, from 2009-01-01
_EPOCH = datetime(2009, 1, 1, tzinfo=timezone.utc)
_EPOCH_DATE = _EPOCH.date()


def _calendar(index: str) -> Tuple[str, int, int]:
    calendar = _CALENDARS.get(index)
    if calendar is None:
        raise ValueError(f"{index} is not a date-based index")
    return calendar


def _index_to_date(index: str, i: int) -> Union[date, datetime]:
    """The start of bucket `i`: a UTC datetime for sub-daily indexes, a date otherwise. Raises
    ValueError for an index without a date calendar."""
    unit, size, phase = _calendar(index)
    if unit == 'second':
        return _EPOCH + timedelta(seconds=i * size)
    if unit == 'day':
        return _EPOCH_DATE + timedelta(days=i * size - phase)
    months = i * size - phase if i else 0
    return date(2009 + months // 12, months % 12 + 1, 1)


def _date_to_index(index: str, d: Union[date, datetime]) -> int:
    """Returns the floor index (latest index whose date is <= the given date).
    For sub-day indexes (minute*, hour*), a plain date is treated as midnight UTC.
    Raises ValueError for dates before the index's first date (2009-01-01; day3's is 2008-12-31, week1's 2008-12-29).
    """
    unit, size, phase = _calendar(index)
    if unit == 'second':
        if isinstance(d, datetime):
            dt = d if d.tzinfo else d.replace(tzinfo=timezone.utc)
        else:
            dt = datetime(d.year, d.month, d.day, tzinfo=timezone.utc)
        if dt < _EPOCH:
            raise ValueError("date is before the first index (2009-01-01)")
        return int((dt - _EPOCH).total_seconds()) // size
    if isinstance(d, datetime):
        d = (d.astimezone(timezone.utc) if d.tzinfo else d).date()
    first = _index_to_date(index, 0)
    if d < first:
        raise ValueError(f"date is before the first {index} date ({first})")
    if unit == 'day':
        return ((d - _EPOCH_DATE).days + phase) // size
    return ((d.year - 2009) * 12 + d.month - 1 + phase) // size


@dataclass
class SeriesData(Generic[T]):
    """Always int-indexed; a value is None where it is missing or undefined."""
    version: int
    index: Index
    type: str
    start: int
    end: int
    data: List[T]

    @property
    def is_date_based(self) -> bool:
        return self.index in _CALENDARS

    def indexes(self) -> List[int]:
        return list(range(self.start, self.end))

    def keys(self) -> List[int]:
        """Alias for indexes()."""
        return self.indexes()

    def items(self) -> List[Tuple[int, T]]:
        return list(zip(range(self.start, self.end), self.data))

    def to_dict(self) -> Dict[int, T]:
        return dict(zip(range(self.start, self.end), self.data))

    def __iter__(self) -> Iterator[Tuple[int, T]]:
        return zip(range(self.start, self.end), self.data)

    def __len__(self) -> int:
        return len(self.data)

    def to_polars(self) -> pl.DataFrame:
        """Convert to Polars DataFrame with 'index' and 'value' columns."""
        try:
            import polars as pl  # type: ignore[import-not-found]
        except ImportError:
            raise ImportError("polars is required: pip install polars")
        return pl.DataFrame({"index": self.indexes(), "value": self.data})

    def to_pandas(self) -> pd.DataFrame:
        """Convert to Pandas DataFrame with 'index' and 'value' columns."""
        try:
            import pandas as pd  # type: ignore[import-not-found]
        except ImportError:
            raise ImportError("pandas is required: pip install pandas")
        return pd.DataFrame({"index": self.indexes(), "value": self.data})


@dataclass
class DateSeriesData(SeriesData[T]):
    def dates(self) -> List[Union[date, datetime]]:
        """Get dates for the index range. Returns datetime for sub-daily indexes, date for daily+."""
        return [_index_to_date(self.index, i) for i in range(self.start, self.end)]

    def date_items(self) -> List[Tuple[Union[date, datetime], T]]:
        return list(zip(self.dates(), self.data))

    def to_date_dict(self) -> Dict[Union[date, datetime], T]:
        return dict(zip(self.dates(), self.data))

    def to_polars(self, with_dates: bool = True) -> pl.DataFrame:
        """Columns 'date' and 'value', or 'index' and 'value' with with_dates=False."""
        try:
            import polars as pl  # type: ignore[import-not-found]
        except ImportError:
            raise ImportError("polars is required: pip install polars")
        if with_dates:
            return pl.DataFrame({"date": self.dates(), "value": self.data})
        return pl.DataFrame({"index": self.indexes(), "value": self.data})

    def to_pandas(self, with_dates: bool = True) -> pd.DataFrame:
        """Columns 'date' and 'value', or 'index' and 'value' with with_dates=False."""
        try:
            import pandas as pd  # type: ignore[import-not-found]
        except ImportError:
            raise ImportError("pandas is required: pip install pandas")
        if with_dates:
            return pd.DataFrame({"date": self.dates(), "value": self.data})
        return pd.DataFrame({"index": self.indexes(), "value": self.data})


AnySeriesData = SeriesData[Any]
AnyDateSeriesData = DateSeriesData[Any]


def _series_data(raw: Dict[str, Any]) -> AnySeriesData:
    """Series data with its helpers (and dates, for a date-based index)."""
    return (DateSeriesData if raw['index'] in _CALENDARS else SeriesData)(**raw)


def _contiguous(key: slice) -> slice:
    """Series slices select a contiguous range: a step is rejected, not ignored."""
    if key.step not in (None, 1):
        raise ValueError("series slices do not support a step")
    return key


class _EndpointConfig:
    client: BitviewClient
    name: str
    index: Index
    start: Optional[int]
    end: Optional[int]
    limit: Optional[int]

    def __init__(self, client: BitviewClient, name: str, index: Index,
                 start: Optional[int] = None, end: Optional[int] = None,
                 limit: Optional[int] = None):
        self.client = client
        self.name = name
        self.index = index
        self.start = start
        self.end = end
        self.limit = limit

    def path(self) -> str:
        return f"/api/series/{_segment(self.name)}/{self.index}"

    def _build_path(self, format: Optional[str] = None) -> str:
        params: List[Tuple[str, Any]] = []
        if self.start is not None:
            params.append(("start", self.start))
        if self.end is not None:
            params.append(("end", self.end))
        if self.limit is not None:
            params.append(("limit", self.limit))
        if format is not None:
            params.append(("format", format))
        query = urlencode(params)
        path = self.path()
        return f"{path}?{query}" if query else path

    def _new(self, start: Optional[int] = None, end: Optional[int] = None,
             limit: Optional[int] = None) -> _EndpointConfig:
        """Counted selections pass `limit`: the server ends them after resolving a negative start."""
        return _EndpointConfig(self.client, self.name, self.index, start, end, limit)

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
    """Builder with a selection (one item or a range) specified."""

    def __init__(self, config: _EndpointConfig):
        self._config = config

    def fetch(self) -> SeriesData[T]:
        return self._config.get_series()

    def fetch_csv(self) -> str:
        return self._config.get_csv()


class SkippedBuilder(RangeBuilder[T]):
    """After skip(n): take() sets a count; fetch() reads from the skipped position to the end."""

    def take(self, n: int) -> RangeBuilder[T]:
        return RangeBuilder(self._config._new(self._config.start, limit=n))


class DateRangeBuilder(RangeBuilder[T]):
    def fetch(self) -> DateSeriesData[T]:
        return self._config.get_date_series()


class DateSkippedBuilder(SkippedBuilder[T]):
    def take(self, n: int) -> DateRangeBuilder[T]:
        return DateRangeBuilder(self._config._new(self._config.start, limit=n))
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

    def __init__(self, client: BitviewClient, name: str, index: Index):
        self._config = _EndpointConfig(client, name, index)

    def __getitem__(self, key: Union[int, slice]) -> RangeBuilder[T]:
        if isinstance(key, int):
            return RangeBuilder(self._config._new(key, limit=1))
        key = _contiguous(key)
        return RangeBuilder(self._config._new(key.start, key.stop))

    def head(self, n: int = 10) -> RangeBuilder[T]:
        return RangeBuilder(self._config._new(end=n))

    def tail(self, n: int = 10) -> RangeBuilder[T]:
        return RangeBuilder(self._config._new(end=0) if n == 0 else self._config._new(start=-n))

    def skip(self, n: int) -> SkippedBuilder[T]:
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
        return self._config.get_version()

    def path(self) -> str:
        return self._config.path()


class DateSeriesEndpoint(SeriesEndpoint[T]):
    """Builder for series endpoint queries with date-based indexing.

    Accepts dates in __getitem__ and returns DateSeriesData from fetch().

    Examples:
        data = endpoint[date(2020, 1, 1)].fetch()
        data = endpoint[date(2020, 1, 1):date(2023, 1, 1)].fetch()
    """

    def __getitem__(self, key: Union[int, slice, date, datetime]) -> DateRangeBuilder[T]:
        if isinstance(key, (date, datetime)):
            idx = _date_to_index(self._config.index, key)
            return DateRangeBuilder(self._config._new(idx, limit=1))
        if isinstance(key, int):
            return DateRangeBuilder(self._config._new(key, limit=1))
        key = _contiguous(key)
        start, stop = key.start, key.stop
        if isinstance(start, (date, datetime)):
            start = _date_to_index(self._config.index, start)
        if isinstance(stop, (date, datetime)):
            stop = _date_to_index(self._config.index, stop)
        return DateRangeBuilder(self._config._new(start, stop))

    def head(self, n: int = 10) -> DateRangeBuilder[T]:
        return DateRangeBuilder(self._config._new(end=n))

    def tail(self, n: int = 10) -> DateRangeBuilder[T]:
        return DateRangeBuilder(self._config._new(end=0) if n == 0 else self._config._new(start=-n))

    def skip(self, n: int) -> DateSkippedBuilder[T]:
        return DateSkippedBuilder(self._config._new(start=n))

    def fetch(self) -> DateSeriesData[T]:
        """Fetch all data."""
        return self._config.get_date_series()


AnySeriesEndpoint = SeriesEndpoint[Any]
AnyDateSeriesEndpoint = DateSeriesEndpoint[Any]


class SeriesPattern(Protocol[T]):
    @property
    def name(self) -> str:
        ...

    def indexes(self) -> List[str]:
        ...

    def get(self, index: Index) -> Optional[SeriesEndpoint[T]]:
        """Get an endpoint builder for a specific index, if supported."""
        ...


