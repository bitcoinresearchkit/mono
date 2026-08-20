use bitview_traversable::Traversable;
use brk_types::{Timestamp, Year};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use schemars::JsonSchema;
use serde::Serialize;

use super::{CohortName, Filter};

/// Class values
pub const CLASS_VALUES: Class<Year> = Class {
    _2009: Year::new(2009),
    _2010: Year::new(2010),
    _2011: Year::new(2011),
    _2012: Year::new(2012),
    _2013: Year::new(2013),
    _2014: Year::new(2014),
    _2015: Year::new(2015),
    _2016: Year::new(2016),
    _2017: Year::new(2017),
    _2018: Year::new(2018),
    _2019: Year::new(2019),
    _2020: Year::new(2020),
    _2021: Year::new(2021),
    _2022: Year::new(2022),
    _2023: Year::new(2023),
    _2024: Year::new(2024),
    _2025: Year::new(2025),
    _2026: Year::new(2026),
};

/// Class filters
pub const CLASS_FILTERS: Class<Filter> = Class {
    _2009: Filter::Class(CLASS_VALUES._2009),
    _2010: Filter::Class(CLASS_VALUES._2010),
    _2011: Filter::Class(CLASS_VALUES._2011),
    _2012: Filter::Class(CLASS_VALUES._2012),
    _2013: Filter::Class(CLASS_VALUES._2013),
    _2014: Filter::Class(CLASS_VALUES._2014),
    _2015: Filter::Class(CLASS_VALUES._2015),
    _2016: Filter::Class(CLASS_VALUES._2016),
    _2017: Filter::Class(CLASS_VALUES._2017),
    _2018: Filter::Class(CLASS_VALUES._2018),
    _2019: Filter::Class(CLASS_VALUES._2019),
    _2020: Filter::Class(CLASS_VALUES._2020),
    _2021: Filter::Class(CLASS_VALUES._2021),
    _2022: Filter::Class(CLASS_VALUES._2022),
    _2023: Filter::Class(CLASS_VALUES._2023),
    _2024: Filter::Class(CLASS_VALUES._2024),
    _2025: Filter::Class(CLASS_VALUES._2025),
    _2026: Filter::Class(CLASS_VALUES._2026),
};

/// Class names
pub const CLASS_NAMES: Class<CohortName> = Class {
    _2009: CohortName::new("class_2009", "2009", "Class 2009"),
    _2010: CohortName::new("class_2010", "2010", "Class 2010"),
    _2011: CohortName::new("class_2011", "2011", "Class 2011"),
    _2012: CohortName::new("class_2012", "2012", "Class 2012"),
    _2013: CohortName::new("class_2013", "2013", "Class 2013"),
    _2014: CohortName::new("class_2014", "2014", "Class 2014"),
    _2015: CohortName::new("class_2015", "2015", "Class 2015"),
    _2016: CohortName::new("class_2016", "2016", "Class 2016"),
    _2017: CohortName::new("class_2017", "2017", "Class 2017"),
    _2018: CohortName::new("class_2018", "2018", "Class 2018"),
    _2019: CohortName::new("class_2019", "2019", "Class 2019"),
    _2020: CohortName::new("class_2020", "2020", "Class 2020"),
    _2021: CohortName::new("class_2021", "2021", "Class 2021"),
    _2022: CohortName::new("class_2022", "2022", "Class 2022"),
    _2023: CohortName::new("class_2023", "2023", "Class 2023"),
    _2024: CohortName::new("class_2024", "2024", "Class 2024"),
    _2025: CohortName::new("class_2025", "2025", "Class 2025"),
    _2026: CohortName::new("class_2026", "2026", "Class 2026"),
};

#[derive(Debug, Default, Clone, Traversable, Serialize, JsonSchema)]
pub struct Class<T> {
    pub _2009: T,
    pub _2010: T,
    pub _2011: T,
    pub _2012: T,
    pub _2013: T,
    pub _2014: T,
    pub _2015: T,
    pub _2016: T,
    pub _2017: T,
    pub _2018: T,
    pub _2019: T,
    pub _2020: T,
    pub _2021: T,
    pub _2022: T,
    pub _2023: T,
    pub _2024: T,
    pub _2025: T,
    pub _2026: T,
}

define_column_id!(
    ClassId for Class, version = 1 {
        _2009 => _2009,
        _2010 => _2010,
        _2011 => _2011,
        _2012 => _2012,
        _2013 => _2013,
        _2014 => _2014,
        _2015 => _2015,
        _2016 => _2016,
        _2017 => _2017,
        _2018 => _2018,
        _2019 => _2019,
        _2020 => _2020,
        _2021 => _2021,
        _2022 => _2022,
        _2023 => _2023,
        _2024 => _2024,
        _2025 => _2025,
        _2026 => _2026,
    }
);

impl Class<CohortName> {
    pub const fn names() -> &'static Self {
        &CLASS_NAMES
    }
}

impl<T> Class<T> {
    pub fn new<F>(mut create: F) -> Self
    where
        F: FnMut(Filter, &'static str) -> T,
    {
        let f = CLASS_FILTERS;
        let n = CLASS_NAMES;
        Self {
            _2009: create(f._2009, n._2009.id),
            _2010: create(f._2010, n._2010.id),
            _2011: create(f._2011, n._2011.id),
            _2012: create(f._2012, n._2012.id),
            _2013: create(f._2013, n._2013.id),
            _2014: create(f._2014, n._2014.id),
            _2015: create(f._2015, n._2015.id),
            _2016: create(f._2016, n._2016.id),
            _2017: create(f._2017, n._2017.id),
            _2018: create(f._2018, n._2018.id),
            _2019: create(f._2019, n._2019.id),
            _2020: create(f._2020, n._2020.id),
            _2021: create(f._2021, n._2021.id),
            _2022: create(f._2022, n._2022.id),
            _2023: create(f._2023, n._2023.id),
            _2024: create(f._2024, n._2024.id),
            _2025: create(f._2025, n._2025.id),
            _2026: create(f._2026, n._2026.id),
        }
    }

    pub fn try_new<F, E>(mut create: F) -> Result<Self, E>
    where
        F: FnMut(Filter, &'static str) -> Result<T, E>,
    {
        let f = CLASS_FILTERS;
        let n = CLASS_NAMES;
        Ok(Self {
            _2009: create(f._2009, n._2009.id)?,
            _2010: create(f._2010, n._2010.id)?,
            _2011: create(f._2011, n._2011.id)?,
            _2012: create(f._2012, n._2012.id)?,
            _2013: create(f._2013, n._2013.id)?,
            _2014: create(f._2014, n._2014.id)?,
            _2015: create(f._2015, n._2015.id)?,
            _2016: create(f._2016, n._2016.id)?,
            _2017: create(f._2017, n._2017.id)?,
            _2018: create(f._2018, n._2018.id)?,
            _2019: create(f._2019, n._2019.id)?,
            _2020: create(f._2020, n._2020.id)?,
            _2021: create(f._2021, n._2021.id)?,
            _2022: create(f._2022, n._2022.id)?,
            _2023: create(f._2023, n._2023.id)?,
            _2024: create(f._2024, n._2024.id)?,
            _2025: create(f._2025, n._2025.id)?,
            _2026: create(f._2026, n._2026.id)?,
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        [
            &self._2009,
            &self._2010,
            &self._2011,
            &self._2012,
            &self._2013,
            &self._2014,
            &self._2015,
            &self._2016,
            &self._2017,
            &self._2018,
            &self._2019,
            &self._2020,
            &self._2021,
            &self._2022,
            &self._2023,
            &self._2024,
            &self._2025,
            &self._2026,
        ]
        .into_iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [
            &mut self._2009,
            &mut self._2010,
            &mut self._2011,
            &mut self._2012,
            &mut self._2013,
            &mut self._2014,
            &mut self._2015,
            &mut self._2016,
            &mut self._2017,
            &mut self._2018,
            &mut self._2019,
            &mut self._2020,
            &mut self._2021,
            &mut self._2022,
            &mut self._2023,
            &mut self._2024,
            &mut self._2025,
            &mut self._2026,
        ]
        .into_iter()
    }

    pub fn par_iter_mut(&mut self) -> impl ParallelIterator<Item = &mut T>
    where
        T: Send + Sync,
    {
        [
            &mut self._2009,
            &mut self._2010,
            &mut self._2011,
            &mut self._2012,
            &mut self._2013,
            &mut self._2014,
            &mut self._2015,
            &mut self._2016,
            &mut self._2017,
            &mut self._2018,
            &mut self._2019,
            &mut self._2020,
            &mut self._2021,
            &mut self._2022,
            &mut self._2023,
            &mut self._2024,
            &mut self._2025,
            &mut self._2026,
        ]
        .into_par_iter()
    }

    pub fn mut_vec_from_timestamp(&mut self, timestamp: Timestamp) -> Option<&mut T> {
        let year = Year::from(timestamp);
        self.get_mut(year)
    }

    pub fn get_mut(&mut self, year: Year) -> Option<&mut T> {
        match u16::from(year) {
            2009 => Some(&mut self._2009),
            2010 => Some(&mut self._2010),
            2011 => Some(&mut self._2011),
            2012 => Some(&mut self._2012),
            2013 => Some(&mut self._2013),
            2014 => Some(&mut self._2014),
            2015 => Some(&mut self._2015),
            2016 => Some(&mut self._2016),
            2017 => Some(&mut self._2017),
            2018 => Some(&mut self._2018),
            2019 => Some(&mut self._2019),
            2020 => Some(&mut self._2020),
            2021 => Some(&mut self._2021),
            2022 => Some(&mut self._2022),
            2023 => Some(&mut self._2023),
            2024 => Some(&mut self._2024),
            2025 => Some(&mut self._2025),
            2026 => Some(&mut self._2026),
            _ => None,
        }
    }
}
