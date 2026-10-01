#[cfg(feature = "storage")]
use bitview_traversable::Traversable;
use brk_types::{Timestamp, Year};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{CohortId, CohortName};

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

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "storage", derive(Traversable))]
pub struct Class<T> {
    /// Uses UTXOs created in calendar year 2009.
    _2009: T,
    /// Uses UTXOs created in calendar year 2010.
    _2010: T,
    /// Uses UTXOs created in calendar year 2011.
    _2011: T,
    /// Uses UTXOs created in calendar year 2012.
    _2012: T,
    /// Uses UTXOs created in calendar year 2013.
    _2013: T,
    /// Uses UTXOs created in calendar year 2014.
    _2014: T,
    /// Uses UTXOs created in calendar year 2015.
    _2015: T,
    /// Uses UTXOs created in calendar year 2016.
    _2016: T,
    /// Uses UTXOs created in calendar year 2017.
    _2017: T,
    /// Uses UTXOs created in calendar year 2018.
    _2018: T,
    /// Uses UTXOs created in calendar year 2019.
    _2019: T,
    /// Uses UTXOs created in calendar year 2020.
    _2020: T,
    /// Uses UTXOs created in calendar year 2021.
    _2021: T,
    /// Uses UTXOs created in calendar year 2022.
    _2022: T,
    /// Uses UTXOs created in calendar year 2023.
    _2023: T,
    /// Uses UTXOs created in calendar year 2024.
    _2024: T,
    /// Uses UTXOs created in calendar year 2025.
    _2025: T,
    /// Uses UTXOs created in calendar year 2026.
    _2026: T,
}

define_cohort_id!(
    ClassId for Class {
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

impl<T> Class<T> {
    pub fn new(mut create: impl FnMut(CohortId) -> T) -> Self {
        Self::from_fn(|id| create(id.cohort()))
    }

    pub(crate) fn try_new<E>(mut create: impl FnMut(CohortId) -> Result<T, E>) -> Result<Self, E> {
        Self::try_from_fn(|id| create(id.cohort()))
    }

    pub fn mut_vec_from_timestamp(&mut self, timestamp: Timestamp) -> Option<&mut T> {
        let year = Year::from(timestamp);
        self.get_mut(year)
    }

    fn get_mut(&mut self, year: Year) -> Option<&mut T> {
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

impl ClassId {
    pub const fn cohort(self) -> CohortId {
        CohortId::Class(self)
    }
}
