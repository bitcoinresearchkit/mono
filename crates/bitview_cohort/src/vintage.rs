use bitview_primitives::Date;
#[cfg(feature = "storage")]
use bitview_traversable::Traversable;
use brk_types::Timestamp;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{CohortId, CohortName};

pub const VINTAGE_NAMES: Vintage<CohortName> = Vintage {
    _2009: CohortName::new("vintage_2009", "2009", "Vintage 2009"),
    _2010: CohortName::new("vintage_2010", "2010", "Vintage 2010"),
    _2011: CohortName::new("vintage_2011", "2011", "Vintage 2011"),
    _2012: CohortName::new("vintage_2012", "2012", "Vintage 2012"),
    _2013: CohortName::new("vintage_2013", "2013", "Vintage 2013"),
    _2014: CohortName::new("vintage_2014", "2014", "Vintage 2014"),
    _2015: CohortName::new("vintage_2015", "2015", "Vintage 2015"),
    _2016: CohortName::new("vintage_2016", "2016", "Vintage 2016"),
    _2017: CohortName::new("vintage_2017", "2017", "Vintage 2017"),
    _2018: CohortName::new("vintage_2018", "2018", "Vintage 2018"),
    _2019: CohortName::new("vintage_2019", "2019", "Vintage 2019"),
    _2020: CohortName::new("vintage_2020", "2020", "Vintage 2020"),
    _2021: CohortName::new("vintage_2021", "2021", "Vintage 2021"),
    _2022: CohortName::new("vintage_2022", "2022", "Vintage 2022"),
    _2023: CohortName::new("vintage_2023", "2023", "Vintage 2023"),
    _2024: CohortName::new("vintage_2024", "2024", "Vintage 2024"),
    _2025: CohortName::new("vintage_2025", "2025", "Vintage 2025"),
    _2026: CohortName::new("vintage_2026", "2026", "Vintage 2026"),
    _2027: CohortName::new("vintage_2027", "2027", "Vintage 2027"),
};

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "storage", derive(Traversable))]
pub struct Vintage<T> {
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
    /// Uses UTXOs created in calendar year 2027.
    _2027: T,
}

define_cohort_id!(
    VintageId for Vintage {
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
        _2027 => _2027,
    }
);

impl<T> Vintage<T> {
    pub fn get_mut(&mut self, timestamp: Timestamp) -> Option<&mut T> {
        let year = Date::from(timestamp).year();
        let id = year
            .checked_sub(2009)
            .and_then(|offset| VintageId::ALL.get(usize::from(offset)))
            .copied();
        debug_assert!(id.is_some(), "no vintage cohort for {year}");
        id.map(|id| id.select_mut(self))
    }
}

impl VintageId {
    pub const fn cohort(self) -> CohortId {
        CohortId::Vintage(self)
    }
}
