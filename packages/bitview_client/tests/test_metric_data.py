# Tests for SeriesData and DateSeriesData helper methods including polars/pandas conversion
# Run: uv run pytest tests/test_metric_data.py -v

from datetime import date, datetime, timezone, timedelta

import pytest

from bitview_client import SeriesData, DateSeriesData


# ============ Fixtures ============


@pytest.fixture
def day1_metric():
    """DateSeriesData with day1 (date-based, daily)."""
    return DateSeriesData(
        version=1,
        index="day1",
        type="n",
        start=0,
        end=5,
        stamp="2024-01-01T00:00:00Z",
        data=[100, 200, 300, 400, 500],
    )


@pytest.fixture
def height_metric():
    """SeriesData with height (non-date-based)."""
    return SeriesData(
        version=1,
        index="height",
        type="n",
        start=800000,
        end=800005,
        stamp="2024-01-01T00:00:00Z",
        data=[1.5, 2.5, 3.5, 4.5, 5.5],
    )


@pytest.fixture
def month1_metric():
    """DateSeriesData with month1."""
    return DateSeriesData(
        version=1,
        index="month1",
        type="n",
        start=0,
        end=3,
        stamp="2024-01-01T00:00:00Z",
        data=[1000, 2000, 3000],
    )


@pytest.fixture
def hour1_metric():
    """DateSeriesData with hour1 (sub-daily)."""
    return DateSeriesData(
        version=1,
        index="hour1",
        type="n",
        start=0,
        end=3,
        stamp="2024-01-01T00:00:00Z",
        data=[10.0, 20.0, 30.0],
    )


@pytest.fixture
def week1_metric():
    """DateSeriesData with week1."""
    return DateSeriesData(
        version=1,
        index="week1",
        type="n",
        start=0,
        end=3,
        stamp="2024-01-01T00:00:00Z",
        data=[5, 10, 15],
    )


@pytest.fixture
def year1_metric():
    """DateSeriesData with year1."""
    return DateSeriesData(
        version=1,
        index="year1",
        type="n",
        start=0,
        end=3,
        stamp="2024-01-01T00:00:00Z",
        data=[100, 200, 300],
    )


@pytest.fixture
def day3_metric():
    """DateSeriesData with day3."""
    return DateSeriesData(
        version=1,
        index="day3",
        type="n",
        start=0,
        end=3,
        stamp="2024-01-01T00:00:00Z",
        data=[7, 8, 9],
    )


@pytest.fixture
def empty_metric():
    """SeriesData with empty data."""
    return SeriesData(
        version=1,
        index="day1",
        type="n",
        start=5,
        end=5,
        stamp="2024-01-01T00:00:00Z",
        data=[],
    )


# ============ is_date_based ============


class TestIsDateBased:
    def test_day1(self, day1_metric):
        assert day1_metric.is_date_based is True

    def test_month1(self, month1_metric):
        assert month1_metric.is_date_based is True

    def test_hour1(self, hour1_metric):
        assert hour1_metric.is_date_based is True

    def test_week1(self, week1_metric):
        assert week1_metric.is_date_based is True

    def test_year1(self, year1_metric):
        assert year1_metric.is_date_based is True

    def test_day3(self, day3_metric):
        assert day3_metric.is_date_based is True

    def test_height(self, height_metric):
        assert height_metric.is_date_based is False


# ============ SeriesData (int-indexed) ============


class TestSeriesData:
    def test_keys(self, height_metric):
        assert height_metric.keys() == [800000, 800001, 800002, 800003, 800004]

    def test_items(self, height_metric):
        items = height_metric.items()
        assert items[0] == (800000, 1.5)
        assert items[-1] == (800004, 5.5)

    def test_to_dict(self, height_metric):
        d = height_metric.to_dict()
        assert d[800000] == 1.5
        assert d[800004] == 5.5
        assert len(d) == 5

    def test_iter(self, height_metric):
        result = list(height_metric)
        assert result == [
            (800000, 1.5),
            (800001, 2.5),
            (800002, 3.5),
            (800003, 4.5),
            (800004, 5.5),
        ]

    def test_len(self, height_metric):
        assert len(height_metric) == 5

    def test_indexes(self, height_metric):
        assert height_metric.indexes() == [800000, 800001, 800002, 800003, 800004]

    def test_empty_data(self, empty_metric):
        assert len(empty_metric) == 0
        assert empty_metric.keys() == []
        assert empty_metric.items() == []
        assert empty_metric.to_dict() == {}
        assert list(empty_metric) == []
        assert empty_metric.indexes() == []


# ============ DateSeriesData inheritance ============


class TestDateSeriesDataInheritance:
    def test_isinstance(self, day1_metric):
        assert isinstance(day1_metric, DateSeriesData)
        assert isinstance(day1_metric, SeriesData)

    def test_int_keys_inherited(self, day1_metric):
        assert day1_metric.keys() == [0, 1, 2, 3, 4]

    def test_int_items_inherited(self, day1_metric):
        items = day1_metric.items()
        assert items[0] == (0, 100)
        assert items[4] == (4, 500)

    def test_int_iter_inherited(self, day1_metric):
        result = list(day1_metric)
        assert result[0] == (0, 100)

    def test_len_inherited(self, day1_metric):
        assert len(day1_metric) == 5

    def test_indexes_inherited(self, day1_metric):
        assert day1_metric.indexes() == [0, 1, 2, 3, 4]

    def test_to_dict_inherited(self, day1_metric):
        d = day1_metric.to_dict()
        assert d[0] == 100
        assert isinstance(list(d.keys())[0], int)


# ============ _index_to_date conversions ============


class TestIndexToDate:
    """Test date conversion for all index types."""

    def test_day1_genesis(self, day1_metric):
        """Day1 index 0 = 2009-01-03 (genesis)."""
        dates = day1_metric.dates()
        assert dates[0] == date(2009, 1, 3)

    def test_day1_index_one(self, day1_metric):
        """Day1 index 1 = 2009-01-09 (6-day gap after genesis)."""
        dates = day1_metric.dates()
        assert dates[1] == date(2009, 1, 9)

    def test_day1_consecutive(self, day1_metric):
        """Day1 indexes 2+ are consecutive days after index 1."""
        dates = day1_metric.dates()
        assert dates[2] == date(2009, 1, 10)
        assert dates[3] == date(2009, 1, 11)
        assert dates[4] == date(2009, 1, 12)

    def test_day1_returns_date_type(self, day1_metric):
        dates = day1_metric.dates()
        assert type(dates[0]) is date

    def test_month1(self, month1_metric):
        dates = month1_metric.dates()
        assert dates[0] == date(2009, 1, 1)
        assert dates[1] == date(2009, 2, 1)
        assert dates[2] == date(2009, 3, 1)
        assert type(dates[0]) is date

    def test_week1(self, week1_metric):
        dates = week1_metric.dates()
        assert dates[0] == date(2009, 1, 3)  # genesis
        assert dates[1] == date(2009, 1, 10)  # +7 days
        assert dates[2] == date(2009, 1, 17)  # +14 days
        assert type(dates[0]) is date

    def test_year1(self, year1_metric):
        dates = year1_metric.dates()
        assert dates[0] == date(2009, 1, 1)
        assert dates[1] == date(2010, 1, 1)
        assert dates[2] == date(2011, 1, 1)
        assert type(dates[0]) is date

    def test_day3(self, day3_metric):
        dates = day3_metric.dates()
        assert dates[0] == date(2008, 12, 31)  # epoch - 1 day (TradingView-aligned)
        assert dates[1] == date(2009, 1, 3)  # +3 days
        assert dates[2] == date(2009, 1, 6)  # +6 days

    def test_hour1_returns_datetime(self, hour1_metric):
        """Sub-daily indexes return datetime, not date."""
        dates = hour1_metric.dates()
        assert isinstance(dates[0], datetime)
        # hour1 index 0 = epoch (2009-01-01 00:00:00 UTC)
        assert dates[0] == datetime(2009, 1, 1, 0, 0, 0, tzinfo=timezone.utc)
        assert dates[1] == datetime(2009, 1, 1, 1, 0, 0, tzinfo=timezone.utc)
        assert dates[2] == datetime(2009, 1, 1, 2, 0, 0, tzinfo=timezone.utc)


# ============ _date_to_index conversions ============


class TestDateToIndex:
    """Test reverse date-to-index conversion."""

    def test_day1_genesis(self):
        from bitview_client import _date_to_index
        assert _date_to_index("day1", date(2009, 1, 3)) == 0

    def test_day1_before_day_one(self):
        from bitview_client import _date_to_index
        # Dates before day1 1 map to 0
        assert _date_to_index("day1", date(2009, 1, 5)) == 0

    def test_day1_index_one(self):
        from bitview_client import _date_to_index
        assert _date_to_index("day1", date(2009, 1, 9)) == 1

    def test_day1_later(self):
        from bitview_client import _date_to_index
        assert _date_to_index("day1", date(2009, 1, 10)) == 2

    def test_month1(self):
        from bitview_client import _date_to_index
        assert _date_to_index("month1", date(2009, 1, 1)) == 0
        assert _date_to_index("month1", date(2009, 2, 1)) == 1
        assert _date_to_index("month1", date(2010, 1, 1)) == 12

    def test_year1(self):
        from bitview_client import _date_to_index
        assert _date_to_index("year1", date(2009, 1, 1)) == 0
        assert _date_to_index("year1", date(2010, 6, 15)) == 1
        assert _date_to_index("year1", date(2020, 1, 1)) == 11

    def test_week1(self):
        from bitview_client import _date_to_index
        assert _date_to_index("week1", date(2009, 1, 3)) == 0
        assert _date_to_index("week1", date(2009, 1, 10)) == 1

    def test_hour1_with_datetime(self):
        from bitview_client import _date_to_index
        epoch = datetime(2009, 1, 1, tzinfo=timezone.utc)
        assert _date_to_index("hour1", epoch) == 0
        assert _date_to_index("hour1", epoch + timedelta(hours=1)) == 1
        assert _date_to_index("hour1", epoch + timedelta(hours=24)) == 24

    def test_hour1_with_plain_date(self):
        """Plain date is treated as midnight UTC for sub-daily."""
        from bitview_client import _date_to_index
        # 2009-01-01 as date → midnight UTC → index 0
        assert _date_to_index("hour1", date(2009, 1, 1)) == 0
        # 2009-01-02 as date → midnight UTC → 24 hours later
        assert _date_to_index("hour1", date(2009, 1, 2)) == 24

    def test_roundtrip_day1(self):
        """date → index → date roundtrip for day1."""
        from bitview_client import _date_to_index, _index_to_date
        for i in range(10):
            d = _index_to_date("day1", i)
            assert _date_to_index("day1", d) == i

    def test_roundtrip_month1(self):
        from bitview_client import _date_to_index, _index_to_date
        for i in range(24):
            d = _index_to_date("month1", i)
            assert _date_to_index("month1", d) == i

    def test_roundtrip_hour1(self):
        from bitview_client import _date_to_index, _index_to_date
        for i in range(48):
            d = _index_to_date("hour1", i)
            assert _date_to_index("hour1", d) == i


# ============ DateSeriesData date methods ============


class TestDateSeriesDataMethods:
    def test_date_items(self, day1_metric):
        items = day1_metric.date_items()
        assert items[0] == (date(2009, 1, 3), 100)
        assert items[1] == (date(2009, 1, 9), 200)
        assert len(items) == 5

    def test_to_date_dict(self, day1_metric):
        d = day1_metric.to_date_dict()
        assert d[date(2009, 1, 3)] == 100
        assert d[date(2009, 1, 9)] == 200
        assert len(d) == 5
        # Keys should be date objects
        assert type(list(d.keys())[0]) is date

    def test_date_items_sub_daily(self, hour1_metric):
        items = hour1_metric.date_items()
        assert isinstance(items[0][0], datetime)
        assert items[0] == (datetime(2009, 1, 1, 0, 0, 0, tzinfo=timezone.utc), 10.0)

    def test_to_date_dict_sub_daily(self, hour1_metric):
        d = hour1_metric.to_date_dict()
        key = datetime(2009, 1, 1, 0, 0, 0, tzinfo=timezone.utc)
        assert d[key] == 10.0
        assert isinstance(list(d.keys())[0], datetime)


# ============ Polars ============


class TestPolarsConversion:
    @pytest.fixture(autouse=True)
    def check_polars(self):
        pytest.importorskip("polars")

    def test_metric_data_to_polars(self, height_metric):
        import polars as pl
        df = height_metric.to_polars()
        assert isinstance(df, pl.DataFrame)
        assert list(df.columns) == ["index", "value"]
        assert df["index"].to_list() == [800000, 800001, 800002, 800003, 800004]
        assert df["value"].to_list() == [1.5, 2.5, 3.5, 4.5, 5.5]

    def test_date_metric_to_polars_with_dates(self, day1_metric):
        import polars as pl
        df = day1_metric.to_polars()
        assert isinstance(df, pl.DataFrame)
        assert "date" in df.columns
        assert "value" in df.columns
        assert "index" not in df.columns
        assert len(df) == 5
        assert df["value"].to_list() == [100, 200, 300, 400, 500]

    def test_date_metric_to_polars_without_dates(self, day1_metric):
        import polars as pl
        df = day1_metric.to_polars(with_dates=False)
        assert "index" in df.columns
        assert "date" not in df.columns
        assert df["index"].to_list() == [0, 1, 2, 3, 4]

    def test_month1_to_polars(self, month1_metric):
        df = month1_metric.to_polars()
        assert "date" in df.columns
        assert len(df) == 3
        dates = df["date"].to_list()
        assert dates[0] == date(2009, 1, 1)
        assert dates[2] == date(2009, 3, 1)

    def test_sub_daily_to_polars(self, hour1_metric):
        df = hour1_metric.to_polars()
        assert "date" in df.columns
        assert len(df) == 3

    def test_empty_to_polars(self, empty_metric):
        df = empty_metric.to_polars()
        assert len(df) == 0
        assert list(df.columns) == ["index", "value"]


# ============ Pandas ============


class TestPandasConversion:
    @pytest.fixture(autouse=True)
    def check_pandas(self):
        pytest.importorskip("pandas")

    def test_metric_data_to_pandas(self, height_metric):
        import pandas as pd
        df = height_metric.to_pandas()
        assert isinstance(df, pd.DataFrame)
        assert list(df.columns) == ["index", "value"]
        assert df["index"].tolist() == [800000, 800001, 800002, 800003, 800004]
        assert df["value"].tolist() == [1.5, 2.5, 3.5, 4.5, 5.5]

    def test_date_metric_to_pandas_with_dates(self, day1_metric):
        import pandas as pd
        df = day1_metric.to_pandas()
        assert isinstance(df, pd.DataFrame)
        assert "date" in df.columns
        assert "value" in df.columns
        assert len(df) == 5
        assert df["value"].tolist() == [100, 200, 300, 400, 500]

    def test_date_metric_to_pandas_without_dates(self, day1_metric):
        import pandas as pd
        df = day1_metric.to_pandas(with_dates=False)
        assert "index" in df.columns
        assert "date" not in df.columns
        assert df["index"].tolist() == [0, 1, 2, 3, 4]

    def test_month1_to_pandas(self, month1_metric):
        df = month1_metric.to_pandas()
        assert "date" in df.columns
        assert len(df) == 3
        dates = df["date"].tolist()
        assert dates[0] == date(2009, 1, 1)
        assert dates[2] == date(2009, 3, 1)

    def test_sub_daily_to_pandas(self, hour1_metric):
        df = hour1_metric.to_pandas()
        assert "date" in df.columns
        assert len(df) == 3

    def test_empty_to_pandas(self, empty_metric):
        df = empty_metric.to_pandas()
        assert len(df) == 0
        assert list(df.columns) == ["index", "value"]
