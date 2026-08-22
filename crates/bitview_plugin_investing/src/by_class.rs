use bitview_traversable::Traversable;
use brk_types::{Date, Day1};

/// DCA class years
pub const DCA_CLASS_YEARS: ByDcaClass<u16> = ByDcaClass {
    from_2015: 2015,
    from_2016: 2016,
    from_2017: 2017,
    from_2018: 2018,
    from_2019: 2019,
    from_2020: 2020,
    from_2021: 2021,
    from_2022: 2022,
    from_2023: 2023,
    from_2024: 2024,
    from_2025: 2025,
    from_2026: 2026,
};

/// DCA class names
pub const DCA_CLASS_NAMES: ByDcaClass<&'static str> = ByDcaClass {
    from_2015: "from_2015",
    from_2016: "from_2016",
    from_2017: "from_2017",
    from_2018: "from_2018",
    from_2019: "from_2019",
    from_2020: "from_2020",
    from_2021: "from_2021",
    from_2022: "from_2022",
    from_2023: "from_2023",
    from_2024: "from_2024",
    from_2025: "from_2025",
    from_2026: "from_2026",
};

/// Generic wrapper for DCA year class data
#[derive(Clone, Default, Traversable)]
pub struct ByDcaClass<T> {
    /// Uses January 1, 2015 as the strategy start date.
    pub from_2015: T,
    /// Uses January 1, 2016 as the strategy start date.
    pub from_2016: T,
    /// Uses January 1, 2017 as the strategy start date.
    pub from_2017: T,
    /// Uses January 1, 2018 as the strategy start date.
    pub from_2018: T,
    /// Uses January 1, 2019 as the strategy start date.
    pub from_2019: T,
    /// Uses January 1, 2020 as the strategy start date.
    pub from_2020: T,
    /// Uses January 1, 2021 as the strategy start date.
    pub from_2021: T,
    /// Uses January 1, 2022 as the strategy start date.
    pub from_2022: T,
    /// Uses January 1, 2023 as the strategy start date.
    pub from_2023: T,
    /// Uses January 1, 2024 as the strategy start date.
    pub from_2024: T,
    /// Uses January 1, 2025 as the strategy start date.
    pub from_2025: T,
    /// Uses January 1, 2026 as the strategy start date.
    pub from_2026: T,
}

impl<T> ByDcaClass<T> {
    fn try_new<F, E>(mut create: F) -> Result<Self, E>
    where
        F: FnMut(&'static str, u16, Day1) -> Result<T, E>,
    {
        let n = DCA_CLASS_NAMES;
        let y = DCA_CLASS_YEARS;
        Ok(Self {
            from_2015: create(n.from_2015, y.from_2015, Self::day1(y.from_2015))?,
            from_2016: create(n.from_2016, y.from_2016, Self::day1(y.from_2016))?,
            from_2017: create(n.from_2017, y.from_2017, Self::day1(y.from_2017))?,
            from_2018: create(n.from_2018, y.from_2018, Self::day1(y.from_2018))?,
            from_2019: create(n.from_2019, y.from_2019, Self::day1(y.from_2019))?,
            from_2020: create(n.from_2020, y.from_2020, Self::day1(y.from_2020))?,
            from_2021: create(n.from_2021, y.from_2021, Self::day1(y.from_2021))?,
            from_2022: create(n.from_2022, y.from_2022, Self::day1(y.from_2022))?,
            from_2023: create(n.from_2023, y.from_2023, Self::day1(y.from_2023))?,
            from_2024: create(n.from_2024, y.from_2024, Self::day1(y.from_2024))?,
            from_2025: create(n.from_2025, y.from_2025, Self::day1(y.from_2025))?,
            from_2026: create(n.from_2026, y.from_2026, Self::day1(y.from_2026))?,
        })
    }

    fn try_from_class<U, F, E>(class: &ByDcaClass<U>, mut create: F) -> Result<Self, E>
    where
        F: FnMut(&'static str, u16, Day1, &U) -> Result<T, E>,
    {
        let n = DCA_CLASS_NAMES;
        let y = DCA_CLASS_YEARS;
        Ok(Self {
            from_2015: create(
                n.from_2015,
                y.from_2015,
                Self::day1(y.from_2015),
                &class.from_2015,
            )?,
            from_2016: create(
                n.from_2016,
                y.from_2016,
                Self::day1(y.from_2016),
                &class.from_2016,
            )?,
            from_2017: create(
                n.from_2017,
                y.from_2017,
                Self::day1(y.from_2017),
                &class.from_2017,
            )?,
            from_2018: create(
                n.from_2018,
                y.from_2018,
                Self::day1(y.from_2018),
                &class.from_2018,
            )?,
            from_2019: create(
                n.from_2019,
                y.from_2019,
                Self::day1(y.from_2019),
                &class.from_2019,
            )?,
            from_2020: create(
                n.from_2020,
                y.from_2020,
                Self::day1(y.from_2020),
                &class.from_2020,
            )?,
            from_2021: create(
                n.from_2021,
                y.from_2021,
                Self::day1(y.from_2021),
                &class.from_2021,
            )?,
            from_2022: create(
                n.from_2022,
                y.from_2022,
                Self::day1(y.from_2022),
                &class.from_2022,
            )?,
            from_2023: create(
                n.from_2023,
                y.from_2023,
                Self::day1(y.from_2023),
                &class.from_2023,
            )?,
            from_2024: create(
                n.from_2024,
                y.from_2024,
                Self::day1(y.from_2024),
                &class.from_2024,
            )?,
            from_2025: create(
                n.from_2025,
                y.from_2025,
                Self::day1(y.from_2025),
                &class.from_2025,
            )?,
            from_2026: create(
                n.from_2026,
                y.from_2026,
                Self::day1(y.from_2026),
                &class.from_2026,
            )?,
        })
    }

    fn day1(year: u16) -> Day1 {
        Day1::try_from(Date::new(year, 1, 1)).unwrap()
    }
}

pub fn try_new<T, F, E>(create: F) -> Result<ByDcaClass<T>, E>
where
    F: FnMut(&'static str, u16, Day1) -> Result<T, E>,
{
    ByDcaClass::try_new(create)
}

pub fn try_from_class<T, U, F, E>(class: &ByDcaClass<U>, create: F) -> Result<ByDcaClass<T>, E>
where
    F: FnMut(&'static str, u16, Day1, &U) -> Result<T, E>,
{
    ByDcaClass::try_from_class(class, create)
}
