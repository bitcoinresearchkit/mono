from datetime import date, datetime, timezone

import pytest

from bitview_client import DateSeriesData, SeriesData, _date_to_index, _index_to_date


def series(index, data=(100, 200, 300), start=0, cls=None):
    cls = cls or (SeriesData if index == "height" else DateSeriesData)
    return cls(version=1, index=index, type="n", start=start, end=start + len(data),
               stamp="2024-01-01T00:00:00Z", data=list(data))


@pytest.mark.parametrize("metric", [
    series("height", [1.5, 2.5, 3.5], 800000),
    series("day1"),
    series("day1", [], 5, cls=SeriesData),
])
def test_integer_mapping_methods(metric):
    expected = list(zip(range(metric.start, metric.end), metric.data))
    assert metric.is_date_based == (metric.index != "height")
    assert metric.keys() == metric.indexes() == [key for key, _ in expected]
    assert metric.items() == list(metric) == expected
    assert metric.to_dict() == dict(expected)
    assert len(metric) == len(expected)


@pytest.mark.parametrize("index, expected", [
    ("day1", [date(2009, 1, 3), date(2009, 1, 9), date(2009, 1, 10),
              date(2009, 1, 11), date(2009, 1, 12)]),
    ("month1", [date(2009, month, 1) for month in (1, 2, 3)]),
    ("week1", [date(2009, 1, day) for day in (3, 10, 17)]),
    ("year1", [date(year, 1, 1) for year in (2009, 2010, 2011)]),
    ("day3", [date(2008, 12, 31), date(2009, 1, 3), date(2009, 1, 6)]),
    ("hour1", [datetime(2009, 1, 1, hour, tzinfo=timezone.utc) for hour in (0, 1, 2)]),
])
def test_calendar_dates_and_mappings(index, expected):
    metric = series(index, range(len(expected)))
    assert metric.is_date_based
    assert metric.dates() == expected
    assert all(type(actual) is type(want) for actual, want in zip(metric.dates(), expected))
    assert metric.date_items() == list(zip(expected, metric.data))
    assert metric.to_date_dict() == dict(zip(expected, metric.data))


@pytest.mark.parametrize("index, value, expected", [
    ("day1", date(2009, 1, 3), 0),
    ("day1", date(2009, 1, 5), 0),  # Genesis gap.
    ("day1", date(2009, 1, 9), 1),
    ("day1", date(2009, 1, 10), 2),
    ("month1", date(2009, 1, 1), 0),
    ("month1", date(2009, 2, 1), 1),
    ("month1", date(2010, 1, 1), 12),
    ("year1", date(2009, 1, 1), 0),
    ("year1", date(2010, 6, 15), 1),
    ("year1", date(2020, 1, 1), 11),
    ("week1", date(2009, 1, 3), 0),
    ("week1", date(2009, 1, 10), 1),
    ("hour1", datetime(2009, 1, 1, tzinfo=timezone.utc), 0),
    ("hour1", datetime(2009, 1, 1, 1, tzinfo=timezone.utc), 1),
    ("hour1", datetime(2009, 1, 2, tzinfo=timezone.utc), 24),
    ("hour1", date(2009, 1, 1), 0),  # Plain dates mean midnight UTC.
    ("hour1", date(2009, 1, 2), 24),
])
def test_date_to_index(index, value, expected):
    assert _date_to_index(index, value) == expected


@pytest.mark.parametrize("index, count", [("day1", 10), ("month1", 24), ("hour1", 48)])
def test_date_index_roundtrip(index, count):
    for i in range(count):
        assert _date_to_index(index, _index_to_date(index, i)) == i


@pytest.mark.parametrize("library", ["pandas", "polars"])
@pytest.mark.parametrize("metric, with_dates, column", [
    (series("height", [1.5, 2.5, 3.5], 800000), True, "index"),
    (series("day1"), True, "date"),
    (series("day1"), False, "index"),
    (series("month1", [1000, 2000, 3000]), True, "date"),
    (series("hour1", [10.0, 20.0, 30.0]), True, "date"),
    (series("day1", [], 5, cls=SeriesData), True, "index"),
])
def test_dataframe_conversion(library, metric, with_dates, column):
    module = pytest.importorskip(library)
    convert = getattr(metric, f"to_{library}")
    frame = convert(with_dates=with_dates) if isinstance(metric, DateSeriesData) else convert()
    assert isinstance(frame, module.DataFrame)
    assert list(frame.columns) == [column, "value"]
    assert len(frame) == len(metric)
    assert list(frame[column]) == (metric.dates() if column == "date" else metric.indexes())
    assert list(frame["value"]) == metric.data
